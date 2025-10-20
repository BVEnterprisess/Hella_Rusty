#!/usr/bin/env python3
"""
QLoRA Training Script for Project Chimera
Trains LoRA adapters for AI agents using quantized base models
"""

import argparse
import json
import os
import torch
from transformers import (
    AutoTokenizer,
    AutoModelForCausalLM,
    TrainingArguments,
    Trainer,
    DataCollatorForLanguageModeling
)
from peft import LoraConfig, get_peft_model, prepare_model_for_kbit_training
from datasets import load_dataset
import bitsandbytes as bnb
import logging

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

def parse_args():
    parser = argparse.ArgumentParser(description='Train QLoRA adapter for Project Chimera')
    parser.add_argument('--base_model', required=True, help='Path to base model')
    parser.add_argument('--output_dir', required=True, help='Output directory for adapter')
    parser.add_argument('--train_file', required=True, help='Training data file (JSONL)')
    parser.add_argument('--validation_file', required=True, help='Validation data file (JSONL)')
    parser.add_argument('--per_device_train_batch_size', type=int, default=1)
    parser.add_argument('--gradient_accumulation_steps', type=int, default=8)
    parser.add_argument('--num_train_epochs', type=int, default=2)
    parser.add_argument('--learning_rate', type=float, default=2e-4)
    parser.add_argument('--lora_r', type=int, default=16)
    parser.add_argument('--lora_alpha', type=int, default=32)
    parser.add_argument('--lora_dropout', type=float, default=0.05)
    parser.add_argument('--bits', type=int, default=4)
    parser.add_argument('--fp16', action='store_true')
    parser.add_argument('--bf16', action='store_true')
    parser.add_argument('--max_seq_length', type=int, default=512)
    parser.add_argument('--logging_steps', type=int, default=100)
    parser.add_argument('--save_steps', type=int, default=500)
    parser.add_argument('--eval_steps', type=int, default=250)
    return parser.parse_args()

def load_and_prepare_dataset(train_file, validation_file, tokenizer, max_seq_length):
    """Load and tokenize dataset"""
    def tokenize_function(examples):
        return tokenizer(
            examples['prompt'],
            text_target=examples['response'],
            truncation=True,
            max_length=max_seq_length,
            padding='max_length'
        )

    # Load datasets
    train_dataset = load_dataset('json', data_files=train_file, split='train')
    val_dataset = load_dataset('json', data_files=validation_file, split='train')

    # Tokenize
    tokenized_train = train_dataset.map(
        tokenize_function,
        batched=True,
        remove_columns=['prompt', 'response']
    )
    tokenized_val = val_dataset.map(
        tokenize_function,
        batched=True,
        remove_columns=['prompt', 'response']
    )

    return tokenized_train, tokenized_val

def find_all_linear_names(model):
    """Find all linear layer names for LoRA"""
    cls = bnb.nn.Linear4bit if args.bits == 4 else (bnb.nn.Linear8bitLt if args.bits == 8 else torch.nn.Linear)
    lora_module_names = set()
    for name, module in model.named_modules():
        if isinstance(module, cls):
            names = name.split('.')
            lora_module_names.add(names[0] if len(names) == 1 else names[-1])

    if 'lm_head' in lora_module_names:  # needed for 16-bit
        lora_module_names.remove('lm_head')
    return list(lora_module_names)

def setup_model_and_tokenizer(base_model_path, bits=4):
    """Load model and tokenizer with quantization"""
    # Load tokenizer
    tokenizer = AutoTokenizer.from_pretrained(base_model_path)
    if tokenizer.pad_token is None:
        tokenizer.pad_token = tokenizer.eos_token

    # Load model with quantization
    if bits == 4:
        model = AutoModelForCausalLM.from_pretrained(
            base_model_path,
            load_in_4bit=True,
            quantization_config=bnb.config.QConfig(
                load_in_4bit=True,
                bnb_4bit_compute_dtype=torch.float16,
                bnb_4bit_use_double_quant=True,
                bnb_4bit_quant_type="nf4"
            ),
            device_map="auto"
        )
    elif bits == 8:
        model = AutoModelForCausalLM.from_pretrained(
            base_model_path,
            load_in_8bit=True,
            device_map="auto"
        )
    else:
        model = AutoModelForCausalLM.from_pretrained(
            base_model_path,
            torch_dtype=torch.float16,
            device_map="auto"
        )

    # Prepare for k-bit training
    model = prepare_model_for_kbit_training(model)

    return model, tokenizer

def main():
    args = parse_args()

    logger.info("Starting QLoRA training...")
    logger.info(f"Base model: {args.base_model}")
    logger.info(f"Output directory: {args.output_dir}")

    # Setup model and tokenizer
    model, tokenizer = setup_model_and_tokenizer(args.base_model, args.bits)

    # Find target modules for LoRA
    target_modules = find_all_linear_names(model)
    logger.info(f"Target modules for LoRA: {target_modules}")

    # Configure LoRA
    lora_config = LoraConfig(
        r=args.lora_r,
        lora_alpha=args.lora_alpha,
        target_modules=target_modules,
        lora_dropout=args.lora_dropout,
        bias="none",
        task_type="CAUSAL_LM"
    )

    # Apply LoRA to model
    model = get_peft_model(model, lora_config)

    # Load and prepare datasets
    train_dataset, val_dataset = load_and_prepare_dataset(
        args.train_file, args.validation_file, tokenizer, args.max_seq_length
    )

    # Data collator
    data_collator = DataCollatorForLanguageModeling(
        tokenizer=tokenizer,
        mlm=False,
    )

    # Training arguments
    training_args = TrainingArguments(
        output_dir=args.output_dir,
        per_device_train_batch_size=args.per_device_train_batch_size,
        gradient_accumulation_steps=args.gradient_accumulation_steps,
        num_train_epochs=args.num_train_epochs,
        learning_rate=args.learning_rate,
        fp16=args.fp16,
        bf16=args.bf16,
        logging_steps=args.logging_steps,
        save_steps=args.save_steps,
        eval_steps=args.eval_steps,
        evaluation_strategy="steps",
        save_strategy="steps",
        load_best_model_at_end=True,
        metric_for_best_model="eval_loss",
        greater_is_better=False,
        save_total_limit=3,
        remove_unused_columns=False,
        report_to="tensorboard"
    )

    # Initialize trainer
    trainer = Trainer(
        model=model,
        args=training_args,
        train_dataset=train_dataset,
        eval_dataset=val_dataset,
        data_collator=data_collator,
    )

    # Train the model
    logger.info("Starting training...")
    trainer.train()

    # Save the model and training metrics
    trainer.save_model(args.output_dir)

    # Save training arguments and config for reproducibility
    training_config = {
        'base_model': args.base_model,
        'lora_config': {
            'r': args.lora_r,
            'lora_alpha': args.lora_alpha,
            'lora_dropout': args.lora_dropout,
            'target_modules': target_modules
        },
        'training_args': vars(args)
    }

    with open(os.path.join(args.output_dir, 'training_config.json'), 'w') as f:
        json.dump(training_config, f, indent=2)

    # Evaluate on validation set
    eval_results = trainer.evaluate()
    logger.info(f"Validation results: {eval_results}")

    # Save evaluation results
    with open(os.path.join(args.output_dir, 'eval_results.json'), 'w') as f:
        json.dump(eval_results, f, indent=2)

    logger.info(f"Training completed. Adapter saved to {args.output_dir}")
    logger.info(f"Model size: {sum(p.numel() for p in model.parameters())} parameters")
    logger.info(f"Trainable parameters: {sum(p.numel() for p in model.parameters() if p.requires_grad)} parameters")

if __name__ == "__main__":
    main()
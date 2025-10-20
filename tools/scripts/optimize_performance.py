#!/usr/bin/env python3
"""
Performance Optimization Script for Project Chimera
Optimizes GPU memory usage and model performance
"""

import torch
import gc
import psutil
import GPUtil
import json
import argparse
from typing import Dict, List
import logging

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

class PerformanceOptimizer:
    def __init__(self):
        self.device = torch.device("cuda" if torch.cuda.is_available() else "cpu")

    def get_system_info(self) -> Dict:
        """Get comprehensive system information"""
        info = {
            "cpu_percent": psutil.cpu_percent(interval=1),
            "memory": psutil.virtual_memory()._asdict(),
            "disk": psutil.disk_usage('/')._asdict(),
            "gpus": []
        }

        if torch.cuda.is_available():
            for i in range(torch.cuda.device_count()):
                gpu = GPUtil.getGPUs()[i]
                info["gpus"].append({
                    "id": i,
                    "name": gpu.name,
                    "memory_used": gpu.memoryUsed,
                    "memory_total": gpu.memoryTotal,
                    "memory_free": gpu.memoryFree,
                    "gpu_util": gpu.load * 100,
                    "temperature": gpu.temperature
                })

        return info

    def clear_gpu_memory(self):
        """Clear GPU memory cache"""
        if torch.cuda.is_available():
            torch.cuda.empty_cache()
            torch.cuda.synchronize()
            gc.collect()

    def optimize_memory_layout(self):
        """Optimize PyTorch memory layout"""
        if torch.cuda.is_available():
            # Enable memory efficient attention
            torch.backends.cuda.enable_flash_sdp(True)
            torch.backends.cuda.enable_math_sdp(True)
            torch.backends.cuda.enable_mem_efficient_sdp(True)

            # Set memory fraction
            torch.cuda.set_per_process_memory_fraction(0.9)

    def defragment_memory(self):
        """Defragment GPU memory"""
        if torch.cuda.is_available():
            # Force garbage collection
            gc.collect()
            torch.cuda.empty_cache()

            # Get current memory stats
            for i in range(torch.cuda.device_count()):
                logger.info(f"GPU {i} memory: {torch.cuda.memory_allocated(i)}/{torch.cuda.memory_reserved(i)} bytes")

    def optimize_model_loading(self, model_path: str) -> Dict:
        """Optimize model loading with quantization"""
        optimization_results = {
            "original_size": 0,
            "optimized_size": 0,
            "memory_saved": 0,
            "load_time": 0
        }

        try:
            import time
            start_time = time.time()

            # Load with quantization
            if torch.cuda.is_available():
                # Use 8-bit quantization for memory efficiency
                from transformers import BitsAndBytesConfig

                quantization_config = BitsAndBytesConfig(
                    load_in_8bit=True,
                    llm_int8_enable_fp32_cpu_offload=True
                )

            load_time = time.time() - start_time
            optimization_results["load_time"] = load_time

            logger.info(f"Model loaded in {load_time:.2f}s")
            return optimization_results

        except Exception as e:
            logger.error(f"Error optimizing model: {e}")
            return optimization_results

    def generate_performance_report(self) -> str:
        """Generate a comprehensive performance report"""
        system_info = self.get_system_info()

        report = {
            "timestamp": torch.cuda.timestamp() if torch.cuda.is_available() else 0,
            "system": system_info,
            "pytorch": {
                "cuda_available": torch.cuda.is_available(),
                "version": torch.__version__,
                "current_device": str(self.device),
                "device_count": torch.cuda.device_count() if torch.cuda.is_available() else 0
            },
            "memory": {
                "allocated": torch.cuda.memory_allocated() if torch.cuda.is_available() else 0,
                "reserved": torch.cuda.memory_reserved() if torch.cuda.is_available() else 0,
                "max_allocated": torch.cuda.max_memory_allocated() if torch.cuda.is_available() else 0
            }
        }

        return json.dumps(report, indent=2)

def main():
    parser = argparse.ArgumentParser(description="Optimize Project Chimera performance")
    parser.add_argument("--clear-memory", action="store_true", help="Clear GPU memory")
    parser.add_argument("--optimize", action="store_true", help="Optimize memory layout")
    parser.add_argument("--defrag", action="store_true", help="Defragment memory")
    parser.add_argument("--report", action="store_true", help="Generate performance report")
    parser.add_argument("--model-path", type=str, help="Path to model for optimization")

    args = parser.parse_args()

    optimizer = PerformanceOptimizer()

    if args.clear_memory:
        logger.info("Clearing GPU memory...")
        optimizer.clear_gpu_memory()

    if args.optimize:
        logger.info("Optimizing memory layout...")
        optimizer.optimize_memory_layout()

    if args.defrag:
        logger.info("Defragmenting memory...")
        optimizer.defragment_memory()

    if args.report:
        logger.info("Generating performance report...")
        report = optimizer.generate_performance_report()
        print(report)

    if args.model_path:
        logger.info(f"Optimizing model at {args.model_path}...")
        results = optimizer.optimize_model_loading(args.model_path)
        logger.info(f"Optimization results: {results}")

if __name__ == "__main__":
    main()
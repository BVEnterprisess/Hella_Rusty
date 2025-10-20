"""Initial database schema

Revision ID: 001
Revises: None
Create Date: 2025-01-20 00:00:00.000000

"""
from alembic import op
import sqlalchemy as sa
from sqlalchemy.dialects import postgresql

# revision identifiers, used by Alembic.
revision = '001'
down_revision = None
branch_labels = None
depends_on = None

def upgrade():
    # Create agents table
    op.create_table('agents',
        sa.Column('id', sa.String(36), primary_key=True),
        sa.Column('name', sa.String(100), nullable=False),
        sa.Column('agent_type', sa.String(50), nullable=False),
        sa.Column('status', sa.String(20), default='inactive'),
        sa.Column('config', postgresql.JSONB, nullable=False),
        sa.Column('created_at', sa.DateTime, server_default=sa.text('now()')),
        sa.Column('updated_at', sa.DateTime, server_default=sa.text('now()')),
        sa.Column('last_seen', sa.DateTime),
        sa.Column('version', sa.String(20), default='1.0.0')
    )

    # Create conversations table
    op.create_table('conversations',
        sa.Column('id', sa.String(36), primary_key=True),
        sa.Column('user_id', sa.String(100)),
        sa.Column('agent_id', sa.String(36), sa.ForeignKey('agents.id')),
        sa.Column('messages', postgresql.JSONB, nullable=False),
        sa.Column('metadata', postgresql.JSONB),
        sa.Column('created_at', sa.DateTime, server_default=sa.text('now()')),
        sa.Column('updated_at', sa.DateTime, server_default=sa.text('now()'))
    )

    # Create model_artifacts table
    op.create_table('model_artifacts',
        sa.Column('id', sa.String(36), primary_key=True),
        sa.Column('name', sa.String(100), nullable=False),
        sa.Column('model_type', sa.String(50), nullable=False),
        sa.Column('file_path', sa.String(500), nullable=False),
        sa.Column('checksum', sa.String(128), nullable=False),
        sa.Column('metadata', postgresql.JSONB),
        sa.Column('is_active', sa.Boolean, default=False),
        sa.Column('created_at', sa.DateTime, server_default=sa.text('now()')),
        sa.Column('version', sa.String(20), default='1.0.0')
    )

    # Create training_jobs table
    op.create_table('training_jobs',
        sa.Column('id', sa.String(36), primary_key=True),
        sa.Column('model_id', sa.String(36), sa.ForeignKey('model_artifacts.id')),
        sa.Column('status', sa.String(20), default='pending'),
        sa.Column('config', postgresql.JSONB, nullable=False),
        sa.Column('metrics', postgresql.JSONB),
        sa.Column('error_message', sa.Text),
        sa.Column('created_at', sa.DateTime, server_default=sa.text('now()')),
        sa.Column('started_at', sa.DateTime),
        sa.Column('completed_at', sa.DateTime)
    )

    # Create indexes for better performance
    op.create_index('ix_agents_status', 'agents', ['status'])
    op.create_index('ix_agents_type', 'agents', ['agent_type'])
    op.create_index('ix_conversations_user_id', 'conversations', ['user_id'])
    op.create_index('ix_conversations_agent_id', 'conversations', ['agent_id'])
    op.create_index('ix_model_artifacts_active', 'model_artifacts', ['is_active'])
    op.create_index('ix_training_jobs_status', 'training_jobs', ['status'])

def downgrade():
    # Drop indexes
    op.drop_index('ix_training_jobs_status')
    op.drop_index('ix_model_artifacts_active')
    op.drop_index('ix_conversations_agent_id')
    op.drop_index('ix_conversations_user_id')
    op.drop_index('ix_agents_type')
    op.drop_index('ix_agents_status')

    # Drop tables
    op.drop_table('training_jobs')
    op.drop_table('model_artifacts')
    op.drop_table('conversations')
    op.drop_table('agents')
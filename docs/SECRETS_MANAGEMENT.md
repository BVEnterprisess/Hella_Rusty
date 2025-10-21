# Project Chimera - Secrets Management Guide

**Created**: October 21, 2025 (Security Emergency - Week 1)  
**Status**: \ud83d\udea8 **CRITICAL SECURITY REQUIREMENT**  
**Authority**: Executive Mandate - Full Commitment to Option 1

---

## \ud83c\udfaf Executive Summary

This document defines the **mandatory** secrets management practices for Project Chimera. All credentials, API keys, and sensitive configuration must follow these guidelines to prevent security breaches.

**Critical Rule**: **NEVER** commit credentials, passwords, API keys, or any sensitive data to version control.

---

## \ud83d\udea8 Current Security Status

### Issues Fixed (Week 1)
- \u2705 **Removed hardcoded credentials** from `docker-compose.yml`
- \u2705 **Created secure `.env.example`** template with guidance
- \u2705 **Updated `.gitignore`** to prevent accidental commits
- \u2705 **Documented secrets management** procedures

### Previously Exposed (MUST ROTATE)
The following credentials were exposed in git history and **MUST BE ROTATED**:

1. **PostgreSQL Password**: `changeme123` \u2192 **ROTATE IMMEDIATELY**
2. **Redis Password**: `changeme123` \u2192 **ROTATE IMMEDIATELY**
3. **MinIO Root Password**: `changeme123` \u2192 **ROTATE IMMEDIATELY**
4. **Grafana Admin Password**: `changeme123` \u2192 **ROTATE IMMEDIATELY**

**Action**: Generate new strong passwords and update all systems.

---

## \ud83d\udd10 Secrets Management by Environment

### Local Development (.env File)

**Purpose**: Local development on developer workstations

**Setup**:
```bash
# 1. Copy the template
cp .env.example .env

# 2. Generate secure passwords
# Linux/Mac:
openssl rand -base64 32

# Windows PowerShell:
-join ((65..90) + (97..122) + (48..57) | Get-Random -Count 32 | % {[char]$_})

# 3. Edit .env and replace all CHANGE_ME_* placeholders
nano .env  # or vim, code, etc.

# 4. Set restrictive permissions (Linux/Mac only)
chmod 600 .env

# 5. Verify .env is in .gitignore
git check-ignore .env  # Should output: .env
```

**File Structure**:
```
Project-Chimera/
\u251c\u2500\u2500 .env                 \u2190 YOUR LOCAL SECRETS (NEVER COMMIT)
\u251c\u2500\u2500 .env.example         \u2190 Template (safe to commit)
\u251c\u2500\u2500 .gitignore           \u2190 Must include .env
\u2514\u2500\u2500 docker-compose.yml   \u2190 References ${ENV_VAR} only
```

**Security Requirements**:
- \u274c Never commit `.env` to git
- \u2705 Use strong random passwords (32+ characters)
- \u2705 Use different passwords for each service
- \u2705 Store `.env` backup in password manager
- \u2705 Rotate credentials every 90 days

---

### Production (Kubernetes External Secrets Operator)

**Purpose**: Production and staging Kubernetes deployments

**Architecture**:
```
\u250c\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2510\n\u2502   Secrets Manager (Choose One)                       \u2502\n\u2502   - HashiCorp Vault                                  \u2502\n\u2502   - AWS Secrets Manager                              \u2502\n\u2502   - GCP Secret Manager                               \u2502\n\u2502   - Azure Key Vault                                  \u2502\n\u2514\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u252c\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2518\n                       \u2502\n                       \u2502 Syncs every 1 minute\n                       \u2502\n                       \u2502\n\u250c\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2534\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2510\n\u2502   External Secrets Operator (K8s)                    \u2502\n\u2502   - Watches SecretStore resources                    \u2502\n\u2502   - Fetches secrets from provider                    \u2502\n\u2502   - Creates/updates K8s Secrets                      \u2502\n\u2514\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u252c\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2518\n                       \u2502\n                       \u2502 Mounts as env vars\n                       \u2502\n                       \u2502\n\u250c\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2534\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2510\n\u2502   Application Pods                                   \u2502\n\u2502   - Read secrets from environment variables          \u2502\n\u2502   - No secrets in container images                   \u2502\n\u2514\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2518\n```

**Installation**:
```bash\n# 1. Install External Secrets Operator\nhelm repo add external-secrets https://charts.external-secrets.io\nhelm install external-secrets \\\n   external-secrets/external-secrets \\\n    -n external-secrets-system \\\n    --create-namespace\n\n# 2. Verify installation\nkubectl get pods -n external-secrets-system\n```

**Configuration Example (HashiCorp Vault)**:
```yaml\n# k8s/external-secrets/secretstore.yaml\napiVersion: external-secrets.io/v1beta1\nkind: SecretStore\nmetadata:\n  name: vault-backend\n  namespace: chimera\nspec:\n  provider:\n    vault:\n      server: \"https://vault.your-domain.com\"\n      path: \"secret\"\n      version: \"v2\"\n      auth:\n        kubernetes:\n          mountPath: \"kubernetes\"\n          role: \"chimera-app\"\n```

**External Secret Definition**:
```yaml\n# k8s/external-secrets/chimera-secrets.yaml\napiVersion: external-secrets.io/v1beta1\nkind: ExternalSecret\nmetadata:\n  name: chimera-secrets\n  namespace: chimera\nspec:\n  refreshInterval: 1m\n  secretStoreRef:\n    name: vault-backend\n    kind: SecretStore\n  target:\n    name: chimera-secrets\n    creationPolicy: Owner\n  data:\n    # PostgreSQL\n    - secretKey: POSTGRES_PASSWORD\n      remoteRef:\n        key: chimera/production\n        property: postgres_password\n    \n    # Redis\n    - secretKey: REDIS_PASSWORD\n      remoteRef:\n        key: chimera/production\n        property: redis_password\n    \n    # MinIO\n    - secretKey: MINIO_ROOT_PASSWORD\n      remoteRef:\n        key: chimera/production\n        property: minio_root_password\n    \n    # Grafana\n    - secretKey: GRAFANA_ADMIN_PASSWORD\n      remoteRef:\n        key: chimera/production\n        property: grafana_admin_password\n```

**Deployment Configuration**:
```yaml\n# k8s/deployments/agent.yaml (excerpt)\napiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: chimera-agent\nspec:\n  template:\n    spec:\n      containers:\n      - name: agent\n        image: chimera/agent:v1\n        env:\n          # Load secrets from External Secret\n          - name: POSTGRES_PASSWORD\n            valueFrom:\n              secretKeyRef:\n                name: chimera-secrets\n                key: POSTGRES_PASSWORD\n          \n          - name: REDIS_PASSWORD\n            valueFrom:\n              secretKeyRef:\n                name: chimera-secrets\n                key: REDIS_PASSWORD\n```

---

## \ud83d\udee1\ufe0f Security Best Practices

### Password Generation

**Requirements**:
| Secret Type | Min Length | Complexity | Rotation |
|-------------|------------|------------|----------|
| Database Passwords | 32 chars | High | 90 days |
| Admin Passwords | 20 chars | High | 90 days |
| API Keys | 64 chars | High | 180 days |
| JWT Secrets | 64 chars | High | Never (rotate keys) |
| Encryption Keys | 64 chars | Highest | Never (use key rotation) |

**Generation Commands**:
```bash\n# Strong random password (32 chars)\nopenssl rand -base64 32\n\n# Hexadecimal (64 chars)\nopenssl rand -hex 32\n\n# URL-safe (32 chars)\npython3 -c \"import secrets; print(secrets.token_urlsafe(32))\"\n\n# Alphanumeric only (32 chars)\npython3 -c \"import secrets, string; print(''.join(secrets.choice(string.ascii_letters + string.digits) for _ in range(32)))\"\n```

---

### Credential Rotation Procedure

**Schedule**: Every 90 days or immediately upon exposure

**Rotation Steps**:
```bash\n# 1. Generate new credentials\nNEW_POSTGRES_PASSWORD=$(openssl rand -base64 32)\nNEW_REDIS_PASSWORD=$(openssl rand -base64 32)\nNEW_MINIO_PASSWORD=$(openssl rand -base64 32)\n\n# 2. Update secrets manager (production)\nvault kv put secret/chimera/production \\\n  postgres_password=\"$NEW_POSTGRES_PASSWORD\" \\\n  redis_password=\"$NEW_REDIS_PASSWORD\" \\\n  minio_root_password=\"$NEW_MINIO_PASSWORD\"\n\n# 3. Wait for External Secrets Operator to sync (1 minute)\nsleep 60\n\n# 4. Rolling restart of services (zero downtime)\nkubectl rollout restart deployment/chimera-agent -n chimera\nkubectl rollout restart deployment/postgres -n chimera\nkubectl rollout restart deployment/redis -n chimera\n\n# 5. Verify all pods are healthy\nkubectl get pods -n chimera\n\n# 6. Document rotation in security audit log\necho \"$(date): Rotated credentials - postgres, redis, minio\" >> docs/SECURITY_AUDIT_LOG.md\n```

---

### Access Control

**Principle of Least Privilege**:
- Developers: Read-only access to staging secrets\n- DevOps: Read/write access to staging, read-only to production\n- Security Team: Full access to all secrets\n- CI/CD: Service accounts with minimal required permissions\n\n**Audit Logging**:
- All secret access must be logged\n- Alerts on unauthorized access attempts\n- Monthly review of access logs\n- Quarterly access rights review\n\n---

## \ud83d\udeab What NOT to Do\n\n### \u274c NEVER Do This:\n\n```bash\n# DON'T: Commit secrets to git\necho \"POSTGRES_PASSWORD=mypassword\" >> .env\ngit add .env\ngit commit -m \"Add config\"  # \u274c CRITICAL SECURITY BREACH\n\n# DON'T: Hard-code credentials\nPOSTGRES_URL=\"postgresql://user:password123@localhost\"  # \u274c INSECURE\n\n# DON'T: Send secrets via Slack/email\n\"Hey, the production password is abc123\"  # \u274c INSECURE\n\n# DON'T: Use weak passwords\nPOSTGRES_PASSWORD=\"password\"  # \u274c EASILY CRACKED\n\n# DON'T: Reuse passwords across services\nPOSTGRES_PASSWORD=\"mypass123\"\nREDIS_PASSWORD=\"mypass123\"  # \u274c SINGLE POINT OF FAILURE\n\n# DON'T: Store secrets in CI/CD logs\necho \"Database password: $DB_PASSWORD\"  # \u274c LOGGED IN PLAIN TEXT\n\n# DON'T: Include secrets in Docker images\nENV POSTGRES_PASSWORD=secret  # \u274c BAKED INTO IMAGE\n```\n\n---\n\n## \u2705 Incident Response\n\n### If Credentials Are Exposed\n\n**Immediate Actions** (within 1 hour):\n1. \ud83d\udea8 **Rotate ALL exposed credentials immediately**\n2. \ud83d\udea8 **Review access logs for unauthorized access**\n3. \ud83d\udea8 **Notify security team**\n4. \ud83d\udea8 **Document incident in security audit log**\n\n**Follow-up Actions** (within 24 hours):\n1. Investigate how exposure occurred\n2. Implement preventive measures\n3. Review and update security procedures\n4. Conduct team training if needed\n\n**Example Rotation Script**:\n```bash\n#!/bin/bash\n# emergency-rotate.sh\n\nset -e\n\necho \"\ud83d\udea8 EMERGENCY CREDENTIAL ROTATION\"\n\n# Generate new credentials\nNEW_POSTGRES_PASSWORD=$(openssl rand -base64 32)\nNEW_REDIS_PASSWORD=$(openssl rand -base64 32)\nNEW_MINIO_PASSWORD=$(openssl rand -base64 32)\n\n# Update production secrets\nvault kv put secret/chimera/production \\\n  postgres_password=\"$NEW_POSTGRES_PASSWORD\" \\\n  redis_password=\"$NEW_REDIS_PASSWORD\" \\\n  minio_root_password=\"$NEW_MINIO_PASSWORD\"\n\n# Trigger immediate sync\nkubectl annotate externalsecret chimera-secrets \\\n  force-sync=$(date +%s) \\\n  -n chimera\n\n# Rolling restart\nfor deployment in agent postgres redis minio; do\n  kubectl rollout restart deployment/$deployment -n chimera\n  kubectl rollout status deployment/$deployment -n chimera\ndone\n\necho \"\u2705 Emergency rotation complete\"\necho \"$(date): Emergency rotation - all credentials\" >> docs/SECURITY_AUDIT_LOG.md\n```\n\n---\n\n## \ud83d\udcca Monitoring & Alerts\n\n### Security Alerts to Configure\n\n1. **Failed Authentication Attempts**\n   - Alert on 5+ failed attempts within 5 minutes\n   - Auto-block IP after 10 attempts\n\n2. **Credential Access**\n   - Alert on secret access outside business hours\n   - Alert on secret access from unknown locations\n\n3. **Configuration Changes**\n   - Alert on changes to ExternalSecret resources\n   - Alert on changes to SecretStore configuration\n\n4. **Anomalous Behavior**\n   - Alert on unusual volume of secret requests\n   - Alert on secret access from unexpected services\n\n---\n\n## \ud83d\udcdd Compliance & Audit\n\n### Security Audit Log\n\nMaintain `docs/SECURITY_AUDIT_LOG.md` with:\n- Date and time of all credential rotations\n- Who performed the rotation\n- Which credentials were rotated\n- Reason for rotation (scheduled vs emergency)\n\n### Quarterly Security Review\n\n**Checklist**:\n- [ ] Review all access permissions\n- [ ] Verify credential rotation schedule\n- [ ] Audit secrets manager access logs\n- [ ] Review incident response procedures\n- [ ] Update security documentation\n- [ ] Conduct team security training\n\n---\n\n## \ud83d\udc65 Team Training\n\n### Required Training for All Developers\n\n1. **Secrets Management Basics** (30 min)\n   - What are secrets?\n   - Why are they critical?\n   - How to handle them properly\n\n2. **Project Chimera Specific** (30 min)\n   - Local `.env` setup\n   - Production secrets access\n   - Incident response procedures\n\n3. **Hands-on Practice** (30 min)\n   - Set up local environment\n   - Rotate a credential\n   - Respond to simulated exposure\n\n---\n\n## \u2753 FAQ\n\n**Q: Can I commit .env.example?**  \nA: \u2705 Yes! .env.example should be committed as a template, but must not contain actual secrets.\n\n**Q: What if I accidentally committed a secret?**  \nA: \ud83d\udea8 Immediately:\n1. Rotate the exposed credential\n2. Remove from git history using `git filter-branch` or BFG Repo-Cleaner\n3. Notify security team\n4. Document in security audit log\n\n**Q: How do I get access to production secrets?**  \nA: Submit access request to DevOps team with:\n- Business justification\n- Required access level\n- Duration of access needed\n\n**Q: Can I use the same password for staging and production?**  \nA: \u274c NO! Always use different credentials for each environment.\n\n**Q: How do I rotate credentials without downtime?**  \nA: Use rolling restarts in Kubernetes. External Secrets Operator will sync new credentials, then rolling restart ensures zero downtime.\n\n---\n\n## \ud83d\udcde Support & Escalation\n\n### Security Issues\n- **Email**: security@project-chimera.com (monitored 24/7)\n- **Slack**: #security-alerts channel\n- **On-call**: PagerDuty \"Security Team\"\n\n### Secrets Management Questions\n- **Email**: devops@project-chimera.com\n- **Slack**: #devops channel\n- **Documentation**: This file\n\n---\n\n## \ud83d\udcc5 Change Log\n\n### Version 1.0 - October 21, 2025\n- \u2705 Initial creation during Week 1 Security Emergency\n- \u2705 Documented current security issues\n- \u2705 Defined secrets management procedures\n- \u2705 Created rotation procedures\n- \u2705 Established best practices\n\n---\n\n**Document Status**: \ud83d\udea8 **ACTIVE** - Security Critical  \n**Last Updated**: October 21, 2025  \n**Next Review**: November 21, 2025  \n**Owner**: DevOps & Security Teams  \n**Classification**: Internal - Security Sensitive\n
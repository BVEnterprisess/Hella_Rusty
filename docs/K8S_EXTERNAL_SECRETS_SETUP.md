# Kubernetes External Secrets Operator - Production Setup Guide

**Date**: October 21, 2025  
**Project**: Project Chimera  
**Purpose**: Secure secrets management for Kubernetes production deployment

---

## 📋 Overview

The **External Secrets Operator (ESO)** synchronizes secrets from external secret stores (AWS Secrets Manager, Azure Key Vault, Google Secret Manager, HashiCorp Vault, etc.) into Kubernetes Secrets.

**Benefits**:
- ✅ Secrets never stored in git or ConfigMaps
- ✅ Centralized secret management and rotation
- ✅ Audit trail for all secret access
- ✅ Fine-grained access control via IAM/RBAC
- ✅ Automatic rotation and synchronization

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────┐
│  External Secret Store                  │
│  (AWS Secrets Manager, Vault, etc.)     │
│                                         │
│  ┌─────────────────────────────────┐   │
│  │ chimera/postgres-password       │   │
│  │ chimera/redis-password          │   │
│  │ chimera/minio-credentials       │   │
│  │ chimera/jwt-secret              │   │
│  └─────────────────────────────────┘   │
└──────────────────┬──────────────────────┘
                   │
                   │ API Call (IAM Auth)
                   │
                   ▼
┌─────────────────────────────────────────┐
│  Kubernetes Cluster                     │
│                                         │
│  ┌───────────────────────────────────┐ │
│  │ External Secrets Operator         │ │
│  │  - Watches ExternalSecret CRDs    │ │
│  │  - Fetches from secret store      │ │
│  │  - Creates K8s Secrets            │ │
│  └───────────────┬───────────────────┘ │
│                  │                      │
│                  ▼                      │
│  ┌───────────────────────────────────┐ │
│  │ Kubernetes Secrets (Auto-created) │ │
│  │  - chimera-postgres-secret        │ │
│  │  - chimera-redis-secret           │ │
│  │  - chimera-minio-secret           │ │
│  └───────────────┬───────────────────┘ │
│                  │                      │
│                  │ Mount as env/volume  │
│                  ▼                      │
│  ┌───────────────────────────────────┐ │
│  │ Application Pods                  │ │
│  │  - agent                          │ │
│  │  - trainer                        │ │
│  │  - postgres                       │ │
│  └───────────────────────────────────┘ │
└─────────────────────────────────────────┘
```

---

## 🚀 Installation

### Prerequisites

- Kubernetes cluster (1.19+)
- `kubectl` configured
- `helm` 3.0+
- Secret store (AWS Secrets Manager, Vault, etc.)
- IAM permissions for secret access

### Step 1: Install External Secrets Operator

```bash
# Add Helm repository
helm repo add external-secrets https://charts.external-secrets.io
helm repo update

# Install the operator
helm install external-secrets \
  external-secrets/external-secrets \
  -n external-secrets-system \
  --create-namespace \
  --set installCRDs=true

# Verify installation
kubectl get pods -n external-secrets-system
```

---

## 🔐 Secret Store Configuration

### Option 1: AWS Secrets Manager

#### 1.1 Create Secrets in AWS

```bash
# Create secrets in AWS Secrets Manager
aws secretsmanager create-secret \
  --name chimera/postgres-password \
  --secret-string '{"password":"YOUR_SECURE_PASSWORD_HERE"}' \
  --region us-east-1

aws secretsmanager create-secret \
  --name chimera/redis-password \
  --secret-string '{"password":"YOUR_SECURE_PASSWORD_HERE"}' \
  --region us-east-1

aws secretsmanager create-secret \
  --name chimera/minio-credentials \
  --secret-string '{"username":"chimera-admin","password":"YOUR_SECURE_PASSWORD_HERE"}' \
  --region us-east-1

aws secretsmanager create-secret \
  --name chimera/jwt-secret \
  --secret-string '{"secret":"YOUR_SECURE_JWT_SECRET_HERE"}' \
  --region us-east-1

aws secretsmanager create-secret \
  --name chimera/grafana-admin \
  --secret-string '{"password":"YOUR_SECURE_PASSWORD_HERE"}' \
  --region us-east-1
```

#### 1.2 Create IAM Role for ESO

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "secretsmanager:GetSecretValue",
        "secretsmanager:DescribeSecret"
      ],
      "Resource": "arn:aws:secretsmanager:us-east-1:*:secret:chimera/*"
    }
  ]
}
```

#### 1.3 Create SecretStore Resource

Save as `k8s/secret-store-aws.yaml`:

```yaml
apiVersion: external-secrets.io/v1beta1
kind: SecretStore
metadata:
  name: aws-secretsmanager
  namespace: chimera
spec:
  provider:
    aws:
      service: SecretsManager
      region: us-east-1
      auth:
        jwt:
          serviceAccountRef:
            name: external-secrets-sa
```

Apply:

```bash
kubectl apply -f k8s/secret-store-aws.yaml
```

---

### Option 2: HashiCorp Vault

#### 2.1 Store Secrets in Vault

```bash
# Enable KV v2 secrets engine
vault secrets enable -path=secret kv-v2

# Store secrets
vault kv put secret/chimera/postgres password="YOUR_SECURE_PASSWORD"
vault kv put secret/chimera/redis password="YOUR_SECURE_PASSWORD"
vault kv put secret/chimera/minio username="chimera-admin" password="YOUR_SECURE_PASSWORD"
vault kv put secret/chimera/jwt secret="YOUR_SECURE_JWT_SECRET"
vault kv put secret/chimera/grafana password="YOUR_SECURE_PASSWORD"
```

#### 2.2 Create Vault Policy

```hcl
path "secret/data/chimera/*" {
  capabilities = ["read"]
}
```

```bash
vault policy write chimera-read-policy chimera-policy.hcl
```

#### 2.3 Create SecretStore Resource

Save as `k8s/secret-store-vault.yaml`:

```yaml
apiVersion: external-secrets.io/v1beta1
kind: SecretStore
metadata:
  name: vault-backend
  namespace: chimera
spec:
  provider:
    vault:
      server: "https://vault.your-domain.com"
      path: "secret"
      version: "v2"
      auth:
        kubernetes:
          mountPath: "kubernetes"
          role: "chimera-role"
          serviceAccountRef:
            name: external-secrets-sa
```

---

## 📝 ExternalSecret Resources

Create ExternalSecret resources to sync secrets from your store to K8s.

### PostgreSQL Secret

Save as `k8s/external-secret-postgres.yaml`:

```yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: postgres-credentials
  namespace: chimera
spec:
  refreshInterval: 1h  # Sync every hour
  secretStoreRef:
    name: aws-secretsmanager  # or vault-backend
    kind: SecretStore
  target:
    name: postgres-secret
    creationPolicy: Owner
  data:
    - secretKey: password
      remoteRef:
        key: chimera/postgres-password
        property: password
```

### Redis Secret

Save as `k8s/external-secret-redis.yaml`:

```yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: redis-credentials
  namespace: chimera
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: aws-secretsmanager
    kind: SecretStore
  target:
    name: redis-secret
    creationPolicy: Owner
  data:
    - secretKey: password
      remoteRef:
        key: chimera/redis-password
        property: password
```

### MinIO Secret

Save as `k8s/external-secret-minio.yaml`:

```yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: minio-credentials
  namespace: chimera
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: aws-secretsmanager
    kind: SecretStore
  target:
    name: minio-secret
    creationPolicy: Owner
  data:
    - secretKey: username
      remoteRef:
        key: chimera/minio-credentials
        property: username
    - secretKey: password
      remoteRef:
        key: chimera/minio-credentials
        property: password
```

### JWT Secret

Save as `k8s/external-secret-jwt.yaml`:

```yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: jwt-secret
  namespace: chimera
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: aws-secretsmanager
    kind: SecretStore
  target:
    name: jwt-secret
    creationPolicy: Owner
  data:
    - secretKey: secret
      remoteRef:
        key: chimera/jwt-secret
        property: secret
```

### Grafana Secret

Save as `k8s/external-secret-grafana.yaml`:

```yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: grafana-credentials
  namespace: chimera
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: aws-secretsmanager
    kind: SecretStore
  target:
    name: grafana-secret
    creationPolicy: Owner
  data:
    - secretKey: admin-password
      remoteRef:
        key: chimera/grafana-admin
        property: password
```

---

## 🚀 Deployment

### Apply All ExternalSecrets

```bash
# Create namespace
kubectl create namespace chimera

# Apply SecretStore
kubectl apply -f k8s/secret-store-aws.yaml

# Apply all ExternalSecrets
kubectl apply -f k8s/external-secret-postgres.yaml
kubectl apply -f k8s/external-secret-redis.yaml
kubectl apply -f k8s/external-secret-minio.yaml
kubectl apply -f k8s/external-secret-jwt.yaml
kubectl apply -f k8s/external-secret-grafana.yaml

# Verify secrets are created
kubectl get externalsecrets -n chimera
kubectl get secrets -n chimera
```

---

## 📦 Using Secrets in Deployments

### Example: PostgreSQL Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: postgres
  namespace: chimera
spec:
  template:
    spec:
      containers:
      - name: postgres
        image: postgres:15-alpine
        env:
        - name: POSTGRES_DB
          value: "chimera"
        - name: POSTGRES_USER
          value: "chimera"
        - name: POSTGRES_PASSWORD
          valueFrom:
            secretKeyRef:
              name: postgres-secret  # Created by ExternalSecret
              key: password
```

### Example: Agent Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: agent
  namespace: chimera
spec:
  template:
    spec:
      containers:
      - name: agent
        image: chimera/agent:latest
        env:
        - name: REDIS_PASSWORD
          valueFrom:
            secretKeyRef:
              name: redis-secret
              key: password
        - name: POSTGRES_PASSWORD
          valueFrom:
            secretKeyRef:
              name: postgres-secret
              key: password
        - name: MINIO_ACCESS_KEY
          valueFrom:
            secretKeyRef:
              name: minio-secret
              key: username
        - name: MINIO_SECRET_KEY
          valueFrom:
            secretKeyRef:
              name: minio-secret
              key: password
        - name: JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: jwt-secret
              key: secret
```

---

## 🔄 Secret Rotation

### Automatic Rotation

ExternalSecret resources automatically sync at the specified `refreshInterval`. When you rotate secrets in the external store, ESO will automatically update the K8s secrets.

### Manual Rotation

```bash
# 1. Rotate secret in external store (e.g., AWS)
aws secretsmanager put-secret-value \
  --secret-id chimera/postgres-password \
  --secret-string '{"password":"NEW_SECURE_PASSWORD"}' \
  --region us-east-1

# 2. Force immediate sync (optional)
kubectl annotate externalsecret postgres-credentials \
  force-sync=$(date +%s) \
  -n chimera

# 3. Restart pods to pick up new secret
kubectl rollout restart deployment postgres -n chimera
```

---

## 🔍 Monitoring & Troubleshooting

### Check ExternalSecret Status

```bash
# View ExternalSecret status
kubectl describe externalsecret postgres-credentials -n chimera

# Check operator logs
kubectl logs -n external-secrets-system \
  -l app.kubernetes.io/name=external-secrets
```

### Common Issues

#### Issue: "SecretStore not ready"
**Solution**: Verify IAM role/permissions and SecretStore configuration

```bash
kubectl get secretstore -n chimera
kubectl describe secretstore aws-secretsmanager -n chimera
```

#### Issue: "Secret not found in store"
**Solution**: Verify secret exists in external store

```bash
# For AWS
aws secretsmanager describe-secret --secret-id chimera/postgres-password

# For Vault
vault kv get secret/chimera/postgres
```

---

## 🔐 Security Best Practices

### 1. IAM/RBAC Permissions
- Use least-privilege principle
- Separate IAM roles per environment (dev/staging/prod)
- Regularly audit permissions

### 2. Encryption
- Enable encryption at rest in secret store
- Use encrypted connections (TLS) to secret store
- Enable K8s secret encryption at rest

### 3. Access Control
- Limit K8s RBAC to ExternalSecret resources
- Use separate namespaces per environment
- Implement network policies

### 4. Audit Logging
- Enable CloudTrail (AWS) or equivalent
- Monitor secret access patterns
- Alert on unauthorized access attempts

### 5. Rotation Schedule
- Rotate secrets every 90 days minimum
- Implement automated rotation where possible
- Test rotation procedures in staging first

---

## 📚 References

- [External Secrets Operator Docs](https://external-secrets.io)
- [AWS Secrets Manager](https://aws.amazon.com/secrets-manager/)
- [HashiCorp Vault](https://www.vaultproject.io/)
- [Azure Key Vault](https://azure.microsoft.com/en-us/services/key-vault/)
- [Google Secret Manager](https://cloud.google.com/secret-manager)

---

## ✅ Implementation Checklist

- [ ] Install External Secrets Operator
- [ ] Choose secret store (AWS/Vault/Azure/GCP)
- [ ] Create secrets in external store
- [ ] Configure IAM/RBAC permissions
- [ ] Create SecretStore resource
- [ ] Create ExternalSecret resources for each secret
- [ ] Update deployments to use new secrets
- [ ] Test secret synchronization
- [ ] Implement secret rotation procedure
- [ ] Configure monitoring and alerts
- [ ] Document for team
- [ ] Train team on usage

---

**Last Updated**: October 21, 2025  
**Owner**: DevOps Team  
**Next Review**: November 21, 2025 (30 days)

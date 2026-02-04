# Racing Coach: Deployment Guide

**Last Updated**: December 2025
**Status**: Production deployment guide for cloud and self-hosted environments

---

## Overview

This guide covers deploying Racing Coach at various scales, from small MVPs (50-100 users) to large-scale production (1000+ users). It includes:

- Capacity planning and resource calculations
- Cloud hosting options with cost comparisons
- Self-hosting guide for homelabs and on-premises deployment
- Production readiness checklist

**Architecture Context**: Racing Coach consists of:

- **Client**: Windows desktop app (pyirsdk) - not deployed, user-installed
- **Server**: FastAPI + AsyncPG + TimescaleDB
- **Web**: React 19 SPA (static files served via nginx or CDN)
- **Database**: PostgreSQL 15+ with TimescaleDB extension

---

## Table of Contents

1. [Capacity Planning](#capacity-planning)
2. [Cloud Hosting Options](#cloud-hosting-options)
3. [Self-Hosting Guide](#self-hosting-guide)
4. [Homelab Deployment](#homelab-deployment)
5. [Production Readiness](#production-readiness)
6. [Monitoring & Maintenance](#monitoring--maintenance)

---

## Capacity Planning

### Storage Requirements

**Per Lap**:

- Telemetry frames: ~3.5 MB (60 Hz × 90 seconds × ~650 bytes/frame)
- Lap metadata: ~200 bytes
- Lap metrics (braking/corners): ~5 KB
- **Total**: ~3.5 MB per lap

**Per Session** (10 laps):

- Telemetry: ~35 MB
- Metadata + metrics: ~55 KB
- **Total**: ~35 MB per session

**Per User (Monthly)**:

| Usage Pattern        | Sessions/Month | Storage/Month |
| -------------------- | -------------- | ------------- |
| Casual (2/week)      | 8              | ~280 MB       |
| Regular (5/week)     | 20             | ~700 MB       |
| Hardcore (daily)     | 30             | ~1.05 GB      |
| **Weighted Average** | ~25            | ~875 MB       |

### Scaling Tiers

#### Tier 1: MVP (50-100 users)

**Assumptions**:

- 75 users (mixed usage)
- 10-20% concurrent at peak
- ~65 GB total storage (first month)

**Resources Needed**:

- **Server**: 2 vCPU, 4 GB RAM
- **Database**: 2 vCPU, 4 GB RAM, 100 GB storage
- **Concurrent Connections**: 10-15 active sessions
- **API Throughput**: ~50 req/min (bursty lap uploads)

**Cost Estimate (Cloud)**: $30-60/month

---

#### Tier 2: Growth (500 users)

**Assumptions**:

- 500 users (regular to hardcore)
- 50-100 concurrent at peak
- ~375-750 GB storage growth/month
- Retention: 90 days raw telemetry, 1 year aggregated

**Resources Needed**:

- **Server**: 4 vCPU, 8 GB RAM (2-4 instances)
- **Database**: 4 vCPU, 16 GB RAM, 1 TB storage
- **Redis Cache**: 2 GB (session metadata, reference laps)
- **Concurrent Connections**: 50-100 active sessions
- **API Throughput**: ~500 req/min at peak

**Cost Estimate (Cloud)**: $200-400/month
**Cost Estimate (Self-Hosted)**: $0-50/month (electricity + VPS)

---

#### Tier 3: Scale (1000 users)

**Assumptions**:

- 1000 users (active community)
- 100-200 concurrent at peak
- ~750 GB - 1.5 TB storage growth/month

**Resources Needed**:

- **Server**: 8 vCPU, 16 GB RAM (4-8 instances, auto-scaling)
- **Database**: 8 vCPU, 32 GB RAM, 2 TB storage (read replicas for analytics)
- **Redis Cache**: 4 GB
- **CDN**: Cloudflare or CloudFront for web assets
- **Concurrent Connections**: 100-200 active sessions
- **API Throughput**: ~1000 req/min at peak

**Cost Estimate (Cloud)**: $500-800/month
**Cost Estimate (Self-Hosted)**: $50-100/month (dedicated server + VPS)

---

#### Tier 4: Enterprise (5000+ users)

**Assumptions**:

- 5000+ users
- 500+ concurrent at peak
- ~4-8 TB storage growth/month

**Resources Needed**:

- **Server**: 16 vCPU, 32 GB RAM (8-16 instances, horizontal scaling)
- **Database**: TimescaleDB distributed hypertables (multi-node cluster)
- **Redis Cluster**: 16 GB
- **Load Balancer**: AWS ALB or Google Cloud Load Balancing
- **Object Storage**: S3/GCS for archived telemetry (>90 days)
- **ML Service** (if enabled): GPU instances (AWS P3/P4, GCP T4)

**Cost Estimate (Cloud)**: $2,000-5,000/month

---

### Database Growth & Retention Strategy

**TimescaleDB Compression**:

- Raw telemetry: 7-day chunks, compressed after 7 days (~10x reduction)
- Compressed size: ~350 KB per lap (~90% savings)

**Retention Policy**:

```sql
-- Keep raw telemetry for 90 days
SELECT add_retention_policy('telemetry_frames', INTERVAL '90 days');

-- Archive to S3/GCS before deletion (optional)
-- Use pg_dump or TimescaleDB continuous aggregates
```

**Storage Over Time** (1000 users):

| Month | Raw Telemetry | Compressed (<90d) | Total                             |
| ----- | ------------- | ----------------- | --------------------------------- |
| 1     | 1 TB          | 0 GB              | 1 TB                              |
| 3     | 3 TB          | 300 GB            | 3.3 TB                            |
| 6     | 2 TB (90d)    | 1.2 TB            | 3.2 TB (plateau)                  |
| 12    | 2 TB          | 2.4 TB            | 4.4 TB (with year-old aggregates) |

---

## Cloud Hosting Options

### Scale-to-Zero / Budget-Conscious Options

Racing Coach's usage pattern (bursty lap uploads, infrequent reads) makes it ideal for scale-to-zero or pay-per-use platforms.

---

#### Option 1: Fly.io + Timescale Cloud (Recommended for MVP)

**Why This Stack**:

- Fly.io Machines auto-hibernate when idle (scale to zero)
- Pay only for active compute time (~$0.01/hour when active)
- Timescale Cloud's starter tier includes TimescaleDB extension
- Global edge deployment (low latency)

**Setup**:

1. **Server** (Fly.io):

   ```bash
   # Install flyctl
   curl -L https://fly.io/install.sh | sh

   # Launch app
   cd apps/racing-coach-server
   fly launch --name racing-coach-api

   # Set secrets
   fly secrets set DATABASE_URL="postgresql+asyncpg://..."
   fly secrets set JWT_SECRET="your-secret-here"

   # Deploy
   fly deploy
   ```

2. **Database** (Timescale Cloud):
   - Sign up at https://console.cloud.timescale.com/
   - Create service: "racing-coach-db" (Starter tier: 2 vCPU, 4 GB RAM, 100 GB)
   - Copy connection string to Fly secrets

3. **Web** (Cloudflare Pages):
   ```bash
   cd apps/racing-coach-web
   npm run build
   npx wrangler pages deploy dist/
   ```

**Cost Estimate**:

| Component               | Tier                                   | Cost/Month |
| ----------------------- | -------------------------------------- | ---------- |
| Fly.io (server)         | 2 vCPU, 4 GB RAM (shared, ~10% active) | $5-15      |
| Timescale Cloud         | Starter (100 GB)                       | $50        |
| Cloudflare Pages        | Free tier                              | $0         |
| Cloudflare R2 (storage) | 10 GB                                  | $0.15      |
| **Total**               |                                        | **$55-65** |

**Scaling**: Add Fly Machines or increase instance size as needed.

---

#### Option 2: Railway + Neon (True Scale-to-Zero)

**Why This Stack**:

- Railway: Generous free tier, pay-per-use after ($0.000463/GB-s RAM)
- Neon: Serverless PostgreSQL, true scale-to-zero (no TimescaleDB, but acceptable for <500 users)
- Dead-simple deployment (connect GitHub repo)

**Setup**:

1. **Server + Web** (Railway):
   - Connect GitHub: https://railway.app/new
   - Railway auto-detects `docker-compose.prod.yaml`
   - Set environment variables in Railway dashboard

2. **Database** (Neon):
   - Sign up at https://neon.tech
   - Create project: "racing-coach-db"
   - Copy connection string to Railway

**Cost Estimate**:

| Component              | Tier                                  | Cost/Month |
| ---------------------- | ------------------------------------- | ---------- |
| Railway (server + web) | Starter ($5 credit/month, then usage) | $5-20      |
| Neon (database)        | Free tier (3 GB storage) → Pro        | $0-19      |
| **Total**              |                                       | **$5-40**  |

**Limitations**: Neon doesn't support TimescaleDB extension. For <500 users, standard PostgreSQL indexes are sufficient. Beyond 500 users, migrate to Timescale Cloud.

---

#### Option 3: Google Cloud Run + Cloud SQL

**Why This Stack**:

- Cloud Run: True scale-to-zero, pay only per request ($0.40 per million requests)
- Cloud SQL: Managed PostgreSQL with TimescaleDB support (via extensions)
- Generous free tier (2M requests/month, 1 GB egress)

**Setup**:

1. **Server** (Cloud Run):

   ```bash
   gcloud run deploy racing-coach-server \
     --source ./apps/racing-coach-server \
     --region us-central1 \
     --allow-unauthenticated \
     --set-env-vars DATABASE_URL="postgresql+asyncpg://..."
   ```

2. **Database** (Cloud SQL):

   ```bash
   gcloud sql instances create racing-coach-db \
     --database-version=POSTGRES_15 \
     --tier=db-f1-micro \
     --region=us-central1

   # Enable TimescaleDB
   gcloud sql databases patch racing-coach-db \
     --database-flags=shared_preload_libraries=timescaledb
   ```

3. **Web** (Firebase Hosting or Cloud Storage):
   ```bash
   cd apps/racing-coach-web
   npm run build
   firebase deploy
   ```

**Cost Estimate**:

| Component        | Tier                                    | Cost/Month |
| ---------------- | --------------------------------------- | ---------- |
| Cloud Run        | 2M requests (free tier)                 | $0-10      |
| Cloud SQL        | db-f1-micro (0.6 GB RAM, 10 GB storage) | $7         |
| Firebase Hosting | Free tier (10 GB bandwidth)             | $0         |
| **Total**        |                                         | **$7-17**  |

**Scaling**: Cloud Run auto-scales to 100 instances. Cloud SQL can scale to 96 vCPU / 624 GB RAM.

---

#### Option 4: Render (Simplest All-in-One)

**Why This Stack**:

- All-in-one platform (server, database, web, cron jobs)
- Free tier with auto-sleep on inactivity
- Zero config deployment (connect GitHub)

**Setup**:

1. **Sign up**: https://render.com
2. **New Web Service**: Point to `apps/racing-coach-server` (Render auto-detects Docker)
3. **New PostgreSQL**: Create managed database (TimescaleDB extension available)
4. **New Static Site**: Point to `apps/racing-coach-web` (Render auto-builds Vite)

**Cost Estimate**:

| Component          | Tier                             | Cost/Month |
| ------------------ | -------------------------------- | ---------- |
| Render Web Service | Free tier (auto-sleep) → Starter | $0-7       |
| Render PostgreSQL  | Free tier (90 days) → Starter    | $0-7       |
| Render Static Site | Free tier                        | $0         |
| **Total**          |                                  | **$0-14**  |

**Limitations**: Free tier spins down after 15 min inactivity (cold start ~30s). Upgrade to Starter ($7/month) for 24/7 uptime.

---

### Cost Comparison Table (MVP to Scale)

| Stack                  | MVP (50-100 users) | Growth (500 users)    | Scale (1000 users)  |
| ---------------------- | ------------------ | --------------------- | ------------------- |
| **Fly.io + Timescale** | $55-65             | $150-250              | $400-600            |
| **Railway + Neon**     | $5-40              | $100-200 (migrate DB) | N/A (use Timescale) |
| **GCP Cloud Run**      | $7-17              | $100-200              | $300-500            |
| **Render**             | $0-14              | $50-100               | $200-400            |
| **AWS ECS + RDS**      | $60-100            | $200-400              | $600-1000           |
| **Self-Hosted**        | $0-10 (VPS only)   | $0-50                 | $0-100              |

---

### Recommended Cloud Stacks by Budget

| Budget             | Stack                       | Use Case                              |
| ------------------ | --------------------------- | ------------------------------------- |
| **$0-20/month**    | Render Free / Railway Free  | Testing, personal use, <50 users      |
| **$20-100/month**  | Fly.io + Timescale Cloud    | MVP, 100-500 users, scale-to-zero     |
| **$100-300/month** | GCP Cloud Run + Cloud SQL   | Growth, 500-1000 users, auto-scaling  |
| **$300-600/month** | Fly.io + Timescale (scaled) | 1000-2000 users, multi-region         |
| **$600+/month**    | AWS ECS Fargate + RDS       | Enterprise, >2000 users, ML inference |

---

## Self-Hosting Guide

### Prerequisites

**Minimum Requirements**:

- 4 CPU cores
- 8 GB RAM
- 50 GB disk space (grows with telemetry)
- Docker + Docker Compose OR Kubernetes
- Domain name (for HTTPS via Let's Encrypt)

**Recommended Requirements** (500+ users):

- 8+ CPU cores
- 16+ GB RAM
- 500 GB SSD (NVMe recommended for database)
- Kubernetes cluster (k3s, k0s, or full k8s)

---

### Option 1: Docker Compose (Simplest)

**Setup**:

```bash
# Clone repository
git clone https://github.com/sawyer/racing-coach.git
cd racing-coach

# Create environment file
cp .env.example .env
nano .env
# Edit:
#   POSTGRES_PASSWORD=<strong-password>
#   JWT_SECRET=<random-secret>
#   DATABASE_URL=postgresql+asyncpg://postgres:<password>@timescaledb:5432/postgres

# Deploy production stack
docker compose -f docker-compose.prod.yaml up -d

# Check logs
docker compose -f docker-compose.prod.yaml logs -f

# Access
# Server: http://localhost:8000
# Web: http://localhost:3000
# Database: localhost:5432
```

**Production Hardening**:

1. **Reverse Proxy** (nginx or Caddy):

   ```nginx
   # /etc/nginx/sites-available/racing-coach
   server {
       listen 80;
       server_name racingcoach.example.com;
       return 301 https://$server_name$request_uri;
   }

   server {
       listen 443 ssl http2;
       server_name racingcoach.example.com;

       ssl_certificate /etc/letsencrypt/live/racingcoach.example.com/fullchain.pem;
       ssl_certificate_key /etc/letsencrypt/live/racingcoach.example.com/privkey.pem;

       location /api/ {
           proxy_pass http://localhost:8000/api/;
           proxy_set_header Host $host;
           proxy_set_header X-Real-IP $remote_addr;
       }

       location / {
           proxy_pass http://localhost:3000/;
           proxy_set_header Host $host;
       }
   }
   ```

2. **SSL Certificate** (Let's Encrypt):

   ```bash
   sudo apt install certbot python3-certbot-nginx
   sudo certbot --nginx -d racingcoach.example.com
   ```

3. **Firewall**:
   ```bash
   sudo ufw allow 80/tcp
   sudo ufw allow 443/tcp
   sudo ufw enable
   ```

**Updates**:

```bash
cd racing-coach
git pull
docker compose -f docker-compose.prod.yaml pull
docker compose -f docker-compose.prod.yaml up -d
# Migrations run automatically on server startup
```

**Backups**:

```bash
# Database backup
docker exec timescaledb pg_dump -U postgres postgres > backup-$(date +%F).sql

# Restore
docker exec -i timescaledb psql -U postgres postgres < backup-2025-12-03.sql
```

---

### Option 2: Kubernetes (Advanced)

**Note**: Kubernetes manifests are not yet provided in the repository. Below is a reference architecture.

**Cluster Requirements**:

- 3+ nodes (1 master, 2+ workers) OR single-node k3s
- Container runtime: containerd or Docker
- Ingress controller: nginx-ingress or Traefik
- Cert-manager (for Let's Encrypt SSL)

**Namespace**:

```yaml
# namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: racing-coach
```

**TimescaleDB Deployment**:

```yaml
# timescaledb-statefulset.yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: timescaledb
  namespace: racing-coach
spec:
  serviceName: timescaledb
  replicas: 1
  selector:
    matchLabels:
      app: timescaledb
  template:
    metadata:
      labels:
        app: timescaledb
    spec:
      containers:
        - name: timescaledb
          image: timescale/timescaledb:2.18.1-pg17
          env:
            - name: POSTGRES_USER
              value: postgres
            - name: POSTGRES_PASSWORD
              valueFrom:
                secretKeyRef:
                  name: postgres-secret
                  key: password
          ports:
            - containerPort: 5432
              name: postgres
          volumeMounts:
            - name: data
              mountPath: /var/lib/postgresql/data
  volumeClaimTemplates:
    - metadata:
        name: data
      spec:
        accessModes: ["ReadWriteOnce"]
        storageClassName: fast-ssd # Use your cluster's SSD storage class
        resources:
          requests:
            storage: 500Gi
---
apiVersion: v1
kind: Service
metadata:
  name: timescaledb
  namespace: racing-coach
spec:
  ports:
    - port: 5432
      targetPort: 5432
  selector:
    app: timescaledb
  clusterIP: None # Headless service for StatefulSet
```

**Server Deployment**:

```yaml
# server-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: racing-coach-server
  namespace: racing-coach
spec:
  replicas: 2 # Horizontal scaling
  selector:
    matchLabels:
      app: racing-coach-server
  template:
    metadata:
      labels:
        app: racing-coach-server
    spec:
      containers:
        - name: server
          image: ghcr.io/sawyer/racing-coach-server:latest
          env:
            - name: DATABASE_URL
              valueFrom:
                secretKeyRef:
                  name: app-secrets
                  key: database-url
            - name: JWT_SECRET
              valueFrom:
                secretKeyRef:
                  name: app-secrets
                  key: jwt-secret
          ports:
            - containerPort: 8000
          resources:
            requests:
              cpu: 1000m
              memory: 2Gi
            limits:
              cpu: 2000m
              memory: 4Gi
          livenessProbe:
            httpGet:
              path: /api/v1/health
              port: 8000
            initialDelaySeconds: 10
            periodSeconds: 30
---
apiVersion: v1
kind: Service
metadata:
  name: racing-coach-server
  namespace: racing-coach
spec:
  ports:
    - port: 8000
      targetPort: 8000
  selector:
    app: racing-coach-server
  type: ClusterIP
```

**Web Deployment**:

```yaml
# web-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: racing-coach-web
  namespace: racing-coach
spec:
  replicas: 2
  selector:
    matchLabels:
      app: racing-coach-web
  template:
    metadata:
      labels:
        app: racing-coach-web
    spec:
      containers:
        - name: web
          image: ghcr.io/sawyer/racing-coach-web:latest
          ports:
            - containerPort: 80
          resources:
            requests:
              cpu: 100m
              memory: 128Mi
            limits:
              cpu: 500m
              memory: 512Mi
---
apiVersion: v1
kind: Service
metadata:
  name: racing-coach-web
  namespace: racing-coach
spec:
  ports:
    - port: 80
      targetPort: 80
  selector:
    app: racing-coach-web
  type: ClusterIP
```

**Ingress**:

```yaml
# ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: racing-coach-ingress
  namespace: racing-coach
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
spec:
  ingressClassName: nginx
  tls:
    - hosts:
        - racingcoach.example.com
      secretName: racingcoach-tls
  rules:
    - host: racingcoach.example.com
      http:
        paths:
          - path: /api/
            pathType: Prefix
            backend:
              service:
                name: racing-coach-server
                port:
                  number: 8000
          - path: /
            pathType: Prefix
            backend:
              service:
                name: racing-coach-web
                port:
                  number: 80
```

**Secrets**:

```bash
# Create secrets
kubectl create secret generic postgres-secret \
  --from-literal=password='<strong-password>' \
  -n racing-coach

kubectl create secret generic app-secrets \
  --from-literal=database-url='postgresql+asyncpg://postgres:<password>@timescaledb.racing-coach.svc.cluster.local:5432/postgres' \
  --from-literal=jwt-secret='<random-secret>' \
  -n racing-coach
```

**Deploy**:

```bash
kubectl apply -f namespace.yaml
kubectl apply -f timescaledb-statefulset.yaml
kubectl apply -f server-deployment.yaml
kubectl apply -f web-deployment.yaml
kubectl apply -f ingress.yaml
```

**Horizontal Pod Autoscaling**:

```yaml
# hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: racing-coach-server-hpa
  namespace: racing-coach
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: racing-coach-server
  minReplicas: 2
  maxReplicas: 8
  metrics:
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 70
    - type: Resource
      resource:
        name: memory
        target:
          type: Utilization
          averageUtilization: 80
```

---

## Homelab Deployment

### Your Setup: Dell PowerEdge T430 + VPS

**Hardware**:

- **Dell PowerEdge T430**: 192 GB RAM, 2x E5-2683v4 (32 cores total) running Kubernetes
- **VPS**: Provides WAN access without exposing home IP

**Advantages**:

- Ample resources for 1000+ users (192 GB RAM is overkill for this workload)
- Kubernetes already running (reuse existing cluster)
- Zero cloud costs (only VPS for ~$5-15/month)

**Disadvantages**:

- Power consumption (~300-400W idle, ~$40-60/month)
- Internet uplink (ensure 100+ Mbps upload for telemetry uploads)
- Maintenance responsibility (backups, updates, uptime)

---

### Architecture: Homelab + VPS Reverse Proxy

```
┌─────────────────────────────────────────────────────────────┐
│  User (Client)                                              │
└─────────────────────┬───────────────────────────────────────┘
                      │ HTTPS
                      ↓
┌─────────────────────────────────────────────────────────────┐
│  VPS (Reverse Proxy / Ingress)                              │
│  - Public IP: 203.0.113.10                                  │
│  - nginx or Cloudflare Tunnel                               │
│  - SSL termination (Let's Encrypt)                          │
└─────────────────────┬───────────────────────────────────────┘
                      │ WireGuard VPN or SSH Tunnel
                      ↓
┌─────────────────────────────────────────────────────────────┐
│  Homelab (Dell PowerEdge T430)                              │
│  - Private network: 192.168.1.0/24                          │
│  - Kubernetes cluster (k3s or k8s)                          │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  racing-coach namespace                              │   │
│  │  - TimescaleDB (StatefulSet, 500 GB PVC)            │   │
│  │  - Server (Deployment, 2-4 replicas)                │   │
│  │  - Web (Deployment, 2 replicas)                     │   │
│  │  - Ingress: nginx-ingress (internal only)           │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

---

### Setup: VPS Reverse Proxy

#### Option 1: nginx Reverse Proxy (Traditional)

**VPS Setup** (Ubuntu 22.04):

```bash
# Install nginx
sudo apt update && sudo apt install nginx certbot python3-certbot-nginx

# Configure site
sudo nano /etc/nginx/sites-available/racing-coach
```

**nginx Config**:

```nginx
# Upstream to homelab (via WireGuard VPN)
upstream homelab_server {
    server 192.168.100.2:8000;  # Replace with homelab K8s ingress IP
}

upstream homelab_web {
    server 192.168.100.2:80;
}

server {
    listen 80;
    server_name racingcoach.example.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name racingcoach.example.com;

    ssl_certificate /etc/letsencrypt/live/racingcoach.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/racingcoach.example.com/privkey.pem;

    # API proxy
    location /api/ {
        proxy_pass http://homelab_server/api/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # WebSocket support (future)
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";

        # Timeouts (lap uploads can be large)
        proxy_read_timeout 300s;
        proxy_send_timeout 300s;
        client_max_body_size 50M;
    }

    # Web proxy
    location / {
        proxy_pass http://homelab_web/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

**Enable**:

```bash
sudo ln -s /etc/nginx/sites-available/racing-coach /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx

# SSL certificate
sudo certbot --nginx -d racingcoach.example.com
```

**WireGuard VPN** (connect VPS to homelab):

```bash
# Homelab: Install WireGuard
sudo apt install wireguard

# Generate keys
wg genkey | tee privatekey | wg pubkey > publickey

# Configure /etc/wireguard/wg0.conf
[Interface]
Address = 192.168.100.1/24
PrivateKey = <homelab-private-key>
ListenPort = 51820

[Peer]
PublicKey = <vps-public-key>
AllowedIPs = 192.168.100.2/32

# Start
sudo systemctl enable wg-quick@wg0
sudo systemctl start wg-quick@wg0

# VPS: Similar config (192.168.100.2/24, connect to homelab's public IP:51820)
```

---

#### Option 2: Cloudflare Tunnel (Easiest, Zero Config)

**Why Cloudflare Tunnel**:

- No VPS needed (Cloudflare provides the proxy)
- No port forwarding or public IP required
- Zero-trust access (no exposed ports on homelab)
- Free for unlimited bandwidth

**Setup**:

```bash
# Homelab: Install cloudflared
curl -L https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64 -o /usr/local/bin/cloudflared
chmod +x /usr/local/bin/cloudflared

# Authenticate
cloudflared tunnel login

# Create tunnel
cloudflared tunnel create racing-coach

# Configure tunnel
nano ~/.cloudflared/config.yml
```

**Config** (`~/.cloudflared/config.yml`):

```yaml
tunnel: <tunnel-id>
credentials-file: /home/user/.cloudflared/<tunnel-id>.json

ingress:
  - hostname: racingcoach.example.com
    service: http://192.168.1.10:80 # K8s ingress IP (internal)
  - service: http_status:404
```

**Run as Service**:

```bash
cloudflared service install
sudo systemctl start cloudflared
sudo systemctl enable cloudflared
```

**DNS** (Cloudflare dashboard):

- Add CNAME record: `racingcoach.example.com` → `<tunnel-id>.cfargotunnel.com`
- SSL mode: Full (strict)

**Cost**: $0/month (Cloudflare Free tier)

---

### Homelab Kubernetes Deployment

**Use the Kubernetes manifests from "Option 2" above**, with these changes:

1. **Storage Class**: Use your homelab's fast storage (NVMe SSD if available):

   ```yaml
   storageClassName: local-path # For k3s default
   # Or create custom StorageClass for specific disk
   ```

2. **Ingress**: Use internal ingress (Cloudflare Tunnel bypasses it):

   ```yaml
   # No need for external LoadBalancer
   # ClusterIP services are sufficient
   ```

3. **Resource Requests**: Tune for your hardware:

   ```yaml
   # Server pods
   resources:
     requests:
       cpu: 2000m # T430 has 32 cores, be generous
       memory: 4Gi
     limits:
       cpu: 4000m
       memory: 8Gi
   ```

4. **TimescaleDB**: Dedicate a full SSD if possible:
   ```yaml
   volumeClaimTemplates:
     - metadata:
         name: data
       spec:
         accessModes: ["ReadWriteOnce"]
         storageClassName: nvme-local # Your fast storage
         resources:
           requests:
             storage: 2Ti # You have plenty of space
   ```

**Deploy**:

```bash
kubectl apply -f namespace.yaml
kubectl apply -f timescaledb-statefulset.yaml
kubectl apply -f server-deployment.yaml
kubectl apply -f web-deployment.yaml
# Skip ingress.yaml if using Cloudflare Tunnel
```

---

### Homelab Considerations

**Power & Uptime**:

- T430 consumes ~300-400W idle, ~500W under load
- Assuming $0.12/kWh: ~$35-60/month electricity cost
- Consider UPS for power outages (database corruption risk)

**Internet Uplink**:

- Lap upload: ~3.5 MB per lap
- 100 concurrent users uploading: ~350 MB/min at peak
- Minimum uplink: 100 Mbps upload (cable/fiber)
- Consider traffic shaping if sharing bandwidth

**Backups**:

```bash
# Automated daily backups to external drive or cloud
kubectl exec -n racing-coach timescaledb-0 -- pg_dump -U postgres postgres | gzip > /mnt/backup/db-$(date +%F).sql.gz

# Offsite backup (optional)
rclone copy /mnt/backup/ remote:racing-coach-backups/
```

**Monitoring**:

- Prometheus + Grafana for metrics (CPU, memory, disk, API latency)
- Uptime monitoring (UptimeRobot or Healthchecks.io)

---

## Production Readiness

Before deploying to production, address these critical items identified in `/docs/PRODUCTION_READINESS.md` (current score: 5.5/10).

### Authentication & Authorization

**Current State**: No authentication implemented.

**Action Required**: Implement JWT-based auth using FastAPI-users or custom:

```bash
# Install FastAPI-users
cd apps/racing-coach-server
uv add 'fastapi-users[sqlalchemy]'
```

**Implementation**:

1. Create `apps/racing-coach-server/src/racing_coach_server/auth/` module
2. Add User model, registration/login endpoints
3. Protect all routes with `get_current_user()` dependency
4. Filter all database queries by `user_id`

**Estimated Effort**: 1-2 days

---

### Connection Pooling

**Current State**: Using `NullPool` (creates new connection per request).

**Action Required**: Change to `QueuePool` in `apps/racing-coach-server/src/racing_coach_server/database/engine.py`:

```python
engine = create_async_engine(
    settings.database_url,
    echo=settings.debug,
    # poolclass=NullPool,  # REMOVE
    pool_size=10,          # Base connections
    max_overflow=20,       # Additional under load
    pool_timeout=30,       # Wait time for connection
    pool_recycle=1800,     # Recycle after 30 min
)
```

**Estimated Effort**: 10 minutes

---

### CORS Configuration

**Current State**: Default CORS allows all origins.

**Action Required**: Restrict to web dashboard origin in `apps/racing-coach-server/src/racing_coach_server/app.py`:

```python
from fastapi.middleware.cors import CORSMiddleware

app.add_middleware(
    CORSMiddleware,
    allow_origins=[
        "https://racingcoach.example.com",  # Production web
        "http://localhost:3000"             # Development
    ],
    allow_credentials=True,
    allow_methods=["GET", "POST", "PUT", "DELETE"],
    allow_headers=["*"],
)
```

**Estimated Effort**: 5 minutes

---

### Rate Limiting

**Current State**: No rate limiting.

**Action Required**: Add rate limiting middleware:

```bash
uv add slowapi
```

```python
from slowapi import Limiter, _rate_limit_exceeded_handler
from slowapi.util import get_remote_address

limiter = Limiter(key_func=get_remote_address)
app.state.limiter = limiter
app.add_exception_handler(RateLimitExceeded, _rate_limit_exceeded_handler)

# Apply to routes
@router.post("/telemetry/lap")
@limiter.limit("100/minute")  # 100 requests per minute
async def upload_lap(...):
    ...
```

**Estimated Effort**: 1 hour

---

### Secrets Management

**Current State**: Hardcoded defaults in config.py.

**Action Required**: Use environment variables (already partially implemented):

```bash
# .env (for Docker Compose)
POSTGRES_PASSWORD=<strong-random-password>
JWT_SECRET=<random-256-bit-secret>
DATABASE_URL=postgresql+asyncpg://postgres:<password>@timescaledb:5432/postgres

# For Kubernetes, use Secrets (see Kubernetes section)
```

**Generate secrets**:

```bash
# Strong password
openssl rand -base64 32

# JWT secret
openssl rand -hex 64
```

**Estimated Effort**: 15 minutes

---

### Monitoring

**Recommended Stack**: Prometheus + Grafana

**Server Metrics** (add to FastAPI):

```bash
uv add prometheus-fastapi-instrumentator
```

```python
from prometheus_fastapi_instrumentator import Instrumentator

Instrumentator().instrument(app).expose(app)
# Metrics endpoint: http://localhost:8000/metrics
```

**Prometheus Config**:

```yaml
# prometheus.yml
scrape_configs:
  - job_name: "racing-coach-server"
    static_configs:
      - targets: ["racing-coach-server:8000"]
```

**Grafana Dashboards**:

- Pre-built FastAPI dashboard: https://grafana.com/grafana/dashboards/14991
- Custom dashboard for lap uploads, session count, user activity

**Estimated Effort**: 2-4 hours

---

## Monitoring & Maintenance

### Key Metrics to Track

**Application**:

- API request rate (req/min)
- API latency (P50, P95, P99)
- Error rate (% 5xx responses)
- Lap uploads per hour
- Active sessions (WebSocket connections, future)

**Database**:

- Connection pool usage (active/idle)
- Query latency (slow query log >1s)
- Storage usage (GB, growth rate)
- TimescaleDB compression ratio

**Infrastructure**:

- CPU usage (server, database)
- Memory usage
- Disk I/O (IOPS, throughput)
- Network egress (GB/month)

---

### Alerting Thresholds

| Metric          | Warning | Critical | Action                             |
| --------------- | ------- | -------- | ---------------------------------- |
| Error rate      | >1%     | >5%      | Check logs, investigate errors     |
| P95 latency     | >2s     | >5s      | Optimize queries, scale server     |
| Database CPU    | >70%    | >90%     | Scale up database instance         |
| Disk usage      | >80%    | >90%     | Archive old telemetry, add storage |
| Connection pool | >80%    | >95%     | Increase pool size or scale        |

---

### Backup Strategy

**Database**:

- **Frequency**: Daily automated backups (pg_dump or managed service snapshots)
- **Retention**: 30 days rolling backups
- **Offsite**: Copy to S3/GCS/Backblaze B2 (encrypted)

**Docker Compose**:

```bash
#!/bin/bash
# backup.sh (run via cron daily)
DATE=$(date +%F)
docker exec timescaledb pg_dump -U postgres postgres | gzip > /backup/db-$DATE.sql.gz

# Keep 30 days
find /backup/ -name "db-*.sql.gz" -mtime +30 -delete

# Upload to cloud (optional)
rclone copy /backup/db-$DATE.sql.gz remote:racing-coach-backups/
```

**Kubernetes**:

```bash
# CronJob for automated backups
kubectl create -f backup-cronjob.yaml
```

```yaml
# backup-cronjob.yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: db-backup
  namespace: racing-coach
spec:
  schedule: "0 2 * * *" # Daily at 2 AM
  jobTemplate:
    spec:
      template:
        spec:
          containers:
            - name: backup
              image: postgres:15
              command:
                - /bin/sh
                - -c
                - pg_dump -h timescaledb -U postgres postgres | gzip > /backup/db-$(date +%F).sql.gz
              env:
                - name: PGPASSWORD
                  valueFrom:
                    secretKeyRef:
                      name: postgres-secret
                      key: password
              volumeMounts:
                - name: backup
                  mountPath: /backup
          restartPolicy: OnFailure
          volumes:
            - name: backup
              persistentVolumeClaim:
                claimName: backup-pvc
```

---

### Update & Migration Process

**Docker Compose**:

```bash
# Pull latest images
cd racing-coach
git pull
docker compose -f docker-compose.prod.yaml pull

# Restart (migrations run automatically on server startup)
docker compose -f docker-compose.prod.yaml up -d

# Check logs
docker compose -f docker-compose.prod.yaml logs -f racing-coach-server
```

**Kubernetes**:

```bash
# Update image tags
kubectl set image deployment/racing-coach-server server=ghcr.io/sawyer/racing-coach-server:v1.2.0 -n racing-coach

# Rollout status
kubectl rollout status deployment/racing-coach-server -n racing-coach

# Rollback if needed
kubectl rollout undo deployment/racing-coach-server -n racing-coach
```

**Database Migrations**:

```bash
# Server auto-runs migrations on startup (via Alembic)
# To manually run:
docker exec racing-coach-server uv run alembic upgrade head

# Or in Kubernetes:
kubectl exec -n racing-coach deployment/racing-coach-server -- uv run alembic upgrade head
```

---

## Summary

### Quick Start Recommendations

**For Personal Use / Testing (<50 users)**:

- **Deploy**: Render Free tier or Railway Free tier
- **Cost**: $0-20/month
- **Effort**: 30 minutes (connect GitHub, done)

**For MVP / Launch (100-500 users)**:

- **Deploy**: Fly.io + Timescale Cloud + Cloudflare Pages
- **Cost**: $55-150/month
- **Effort**: 2-4 hours (includes DNS, SSL, monitoring setup)

**For Self-Hosting (any scale)**:

- **Deploy**: Docker Compose + nginx + Let's Encrypt
- **Cost**: $0-50/month (VPS + electricity)
- **Effort**: 4-8 hours (includes VPN/tunnel setup, backups, monitoring)

**For Your Homelab (T430 + VPS)**:

- **Deploy**: Kubernetes + Cloudflare Tunnel
- **Cost**: $0/month (sunk cost: electricity)
- **Effort**: 6-10 hours (K8s manifests, tunnel setup, backups, monitoring)

---

## Next Steps

1. **Choose deployment option** (cloud vs self-hosted)
2. **Complete production readiness checklist** (auth, pooling, CORS, rate limiting)
3. **Set up monitoring** (Prometheus + Grafana)
4. **Deploy to staging environment** (test end-to-end)
5. **Deploy to production**
6. **Configure backups and alerting**

**Questions?** See `/docs/ARCHITECTURE.md` for system design details or `/docs/PRODUCTION_READINESS.md` for security audit.

---

**Last Updated**: December 2025
**Maintainers**: Racing Coach Engineering Team
**Version**: 1.0

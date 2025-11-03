# MultiOS Deployment Toolkit and Installation Guides

## Table of Contents
1. [Quick Start Deployment Guide](#quick-start-deployment-guide)
2. [Advanced Installation Options](#advanced-installation-options)
3. [Platform-Specific Guides](#platform-specific-guides)
4. [Configuration Management](#configuration-management)
5. [Integration Setup](#integration-setup)
6. [Troubleshooting Guide](#troubleshooting-guide)

---

## Quick Start Deployment Guide

### System Requirements

#### Minimum Requirements
```
CPU: 4 cores, 2.5 GHz
RAM: 16 GB
Storage: 100 GB available space
Network: 1 Gbps connection
OS: Ubuntu 20.04+ / CentOS 8+ / RHEL 8+
```

#### Recommended Requirements
```
CPU: 8 cores, 3.0 GHz
RAM: 32 GB
Storage: 500 GB SSD
Network: 10 Gbps connection
OS: Ubuntu 22.04 LTS / RHEL 9
```

#### High Availability Configuration
```
Load Balancer: 2 nodes (HA configuration)
Application Servers: 3+ nodes (minimum)
Database Cluster: 3-node PostgreSQL cluster
Cache Layer: Redis Cluster (3+ nodes)
Storage: Distributed file system or cloud storage
```

---

### Automated Installation (Recommended)

#### One-Command Deployment
```bash
curl -sSL https://install.multios.edu | bash -s -- --institution-name "Your Institution Name" --tier silver --email admin@institution.edu
```

#### Installation Script Breakdown
```bash
#!/bin/bash
# MultiOS Educational Quick Install Script

# Configuration
MULTIOS_VERSION="v2.1.0"
INSTALL_DIR="/opt/multios"
CONFIG_DIR="/etc/multios"
LOG_DIR="/var/log/multios"
SERVICE_USER="multios"

# Check system requirements
check_system() {
    echo "Checking system requirements..."
    
    # Check OS version
    if ! grep -E "(Ubuntu 20| CentOS 8| RHEL 8)" /etc/os-release > /dev/null; then
        echo "Error: Unsupported operating system"
        exit 1
    fi
    
    # Check available memory
    available_mem=$(free -g | awk '/^Mem:/{print $2}')
    if [ $available_mem -lt 16 ]; then
        echo "Error: Insufficient memory (16GB minimum required)"
        exit 1
    fi
    
    # Check available disk space
    available_disk=$(df -BG / | awk 'NR==2{print $4}' | sed 's/G//')
    if [ $available_disk -lt 100 ]; then
        echo "Error: Insufficient disk space (100GB minimum required)"
        exit 1
    fi
    
    echo "System requirements check passed"
}

# Install dependencies
install_dependencies() {
    echo "Installing system dependencies..."
    
    if command -v apt-get > /dev/null; then
        # Ubuntu/Debian
        apt-get update
        apt-get install -y postgresql postgresql-contrib redis-server nginx python3 python3-pip nodejs npm
    elif command -v yum > /dev/null; then
        # CentOS/RHEL
        yum install -y postgresql-server postgresql-contrib redis nginx python3 python3-pip nodejs npm
        systemctl enable postgresql
        systemctl start postgresql
    fi
    
    echo "Dependencies installed successfully"
}

# Download and install MultiOS
install_multios() {
    echo "Downloading MultiOS platform..."
    
    cd /tmp
    wget https://releases.multios.edu/${MULTIOS_VERSION}/multios-educational.tar.gz
    tar -xzf multios-educational.tar.gz
    
    echo "Installing MultiOS platform..."
    mkdir -p $INSTALL_DIR $CONFIG_DIR $LOG_DIR
    cp -r multios-educational/* $INSTALL_DIR/
    
    # Set permissions
    chown -R $SERVICE_USER:$SERVICE_USER $INSTALL_DIR $CONFIG_DIR $LOG_DIR
    chmod +x $INSTALL_DIR/bin/*
    
    echo "MultiOS platform installed successfully"
}

# Configure database
configure_database() {
    echo "Configuring database..."
    
    # Initialize PostgreSQL if not already done
    if [ ! -f "/var/lib/pgsql/data/PG_VERSION" ]; then
        sudo -u postgres postgresql-setup initdb
    fi
    
    # Create MultiOS database and user
    sudo -u postgres psql << EOF
CREATE DATABASE multios_edu;
CREATE USER multios_user WITH ENCRYPTED PASSWORD 'multios_secure_password';
GRANT ALL PRIVILEGES ON DATABASE multios_edu TO multios_user;
\q
EOF
    
    echo "Database configured successfully"
}

# Configure nginx
configure_nginx() {
    echo "Configuring web server..."
    
    cat > /etc/nginx/sites-available/multios << EOF
server {
    listen 80;
    server_name your-institution-domain.edu;
    
    location / {
        proxy_pass http://localhost:8080;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
    }
}
EOF
    
    # Enable site
    ln -sf /etc/nginx/sites-available/multios /etc/nginx/sites-enabled/
    rm -f /etc/nginx/sites-enabled/default
    
    systemctl reload nginx
    echo "Web server configured successfully"
}

# Start services
start_services() {
    echo "Starting MultiOS services..."
    
    # Create systemd service
    cat > /etc/systemd/system/multios.service << EOF
[Unit]
Description=MultiOS Educational Platform
After=network.target

[Service]
Type=simple
User=$SERVICE_USER
WorkingDirectory=$INSTALL_DIR
ExecStart=$INSTALL_DIR/bin/multios-server
Restart=always

[Install]
WantedBy=multi-user.target
EOF
    
    systemctl daemon-reload
    systemctl enable multios
    systemctl start multios
    
    echo "Services started successfully"
}

# Main installation process
main() {
    echo "Starting MultiOS Educational Platform installation..."
    
    check_system
    install_dependencies
    install_multios
    configure_database
    configure_nginx
    start_services
    
    echo "MultiOS Educational Platform installed successfully!"
    echo "Access your portal at: http://your-institution-domain.edu"
    echo "Default admin credentials will be displayed after setup completion"
}

# Run installation
main
```

---

## Advanced Installation Options

### Docker Deployment

#### Docker Compose Configuration
```yaml
version: '3.8'

services:
  multios-portal:
    image: multios/educational-portal:v2.1.0
    ports:
      - "80:80"
      - "443:443"
    environment:
      - MULTIOS_ENV=production
      - DB_HOST=postgres
      - DB_NAME=multios_edu
      - DB_USER=multios_user
      - DB_PASSWORD=${DB_PASSWORD}
      - REDIS_HOST=redis
      - SMTP_HOST=${SMTP_HOST}
      - SMTP_USER=${SMTP_USER}
      - SMTP_PASSWORD=${SMTP_PASSWORD}
      - INSTITUTION_NAME=${INSTITUTION_NAME}
      - PARTNERSHIP_TIER=${PARTNERSHIP_TIER}
    volumes:
      - ./config:/app/config:ro
      - ./data:/app/data
      - ./logs:/app/logs
      - /var/run/docker.sock:/var/run/docker.sock
    depends_on:
      - postgres
      - redis
    restart: unless-stopped
    networks:
      - multios-network

  postgres:
    image: postgres:13
    environment:
      - POSTGRES_DB=multios_edu
      - POSTGRES_USER=multios_user
      - POSTGRES_PASSWORD=${DB_PASSWORD}
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./init-scripts:/docker-entrypoint-initdb.d:ro
    restart: unless-stopped
    networks:
      - multios-network

  redis:
    image: redis:6-alpine
    command: redis-server --appendonly yes
    volumes:
      - redis_data:/data
    restart: unless-stopped
    networks:
      - multios-network

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx/nginx.conf:/etc/nginx/nginx.conf:ro
      - ./nginx/sites:/etc/nginx/conf.d:ro
      - ./ssl:/etc/nginx/ssl:ro
    depends_on:
      - multios-portal
    restart: unless-stopped
    networks:
      - multios-network

  backup:
    image: postgres:13
    environment:
      - PGPASSWORD=${DB_PASSWORD}
    volumes:
      - ./backups:/backups
      - postgres_data:/var/lib/postgresql/data:ro
    command: |
      bash -c "
        while true; do
          pg_dump -h postgres -U multios_user multios_edu > /backups/multios_edu_$$(date +%Y%m%d_%H%M%S).sql
          find /backups -name '*.sql' -mtime +7 -delete
          sleep 86400
        done
      "
    depends_on:
      - postgres
    restart: unless-stopped
    networks:
      - multios-network

volumes:
  postgres_data:
  redis_data:

networks:
  multios-network:
    driver: bridge
```

#### Docker Deployment Commands
```bash
# Clone deployment repository
git clone https://github.com/multios/deployment-templates.git
cd deployment-templates/docker-compose

# Configure environment variables
cp .env.example .env
nano .env  # Edit with your institution details

# Start all services
docker-compose up -d

# Check service status
docker-compose ps

# View logs
docker-compose logs -f multios-portal

# Scale application servers
docker-compose up -d --scale multios-portal=3
```

### Kubernetes Deployment

#### Namespace and Configuration
```yaml
# namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: multios-education

---
# configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: multios-config
  namespace: multios-education
data:
  DATABASE_HOST: "postgres-service"
  DATABASE_NAME: "multios_edu"
  DATABASE_USER: "multios_user"
  REDIS_HOST: "redis-service"
  INSTITUTION_NAME: "Your Institution"
  PARTNERSHIP_TIER: "silver"
  MULTIOS_ENV: "production"
```

#### Database Deployment
```yaml
# postgres-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: postgres
  namespace: multios-education
spec:
  replicas: 1
  selector:
    matchLabels:
      app: postgres
  template:
    metadata:
      labels:
        app: postgres
    spec:
      containers:
      - name: postgres
        image: postgres:13
        env:
        - name: POSTGRES_DB
          value: multios_edu
        - name: POSTGRES_USER
          value: multios_user
        - name: POSTGRES_PASSWORD
          valueFrom:
            secretKeyRef:
              name: multios-secrets
              key: database-password
        ports:
        - containerPort: 5432
        volumeMounts:
        - name: postgres-storage
          mountPath: /var/lib/postgresql/data
      volumes:
      - name: postgres-storage
        persistentVolumeClaim:
          claimName: postgres-pvc

---
apiVersion: v1
kind: Service
metadata:
  name: postgres-service
  namespace: multios-education
spec:
  selector:
    app: postgres
  ports:
  - port: 5432
    targetPort: 5432

---
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: postgres-pvc
  namespace: multios-education
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 100Gi
```

#### Application Deployment
```yaml
# multios-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: multios-portal
  namespace: multios-education
spec:
  replicas: 3
  selector:
    matchLabels:
      app: multios-portal
  template:
    metadata:
      labels:
        app: multios-portal
    spec:
      containers:
      - name: multios-portal
        image: multios/educational-portal:v2.1.0
        ports:
        - containerPort: 80
        env:
        - name: DATABASE_HOST
          value: "postgres-service"
        - name: DATABASE_NAME
          value: "multios_edu"
        - name: DATABASE_USER
          value: "multios_user"
        - name: DATABASE_PASSWORD
          valueFrom:
            secretKeyRef:
              name: multios-secrets
              key: database-password
        - name: INSTITUTION_NAME
          value: "Your Institution"
        - name: PARTNERSHIP_TIER
          value: "silver"
        resources:
          requests:
            memory: "1Gi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "1000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 80
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 80
          initialDelaySeconds: 5
          periodSeconds: 5

---
apiVersion: v1
kind: Service
metadata:
  name: multios-service
  namespace: multios-education
spec:
  selector:
    app: multios-portal
  ports:
  - port: 80
    targetPort: 80
  type: ClusterIP

---
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: multios-ingress
  namespace: multios-education
  annotations:
    kubernetes.io/ingress.class: nginx
    cert-manager.io/cluster-issuer: letsencrypt-prod
spec:
  tls:
  - hosts:
    - multios.your-institution.edu
    secretName: multios-tls
  rules:
  - host: multios.your-institution.edu
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: multios-service
            port:
              number: 80
```

#### Kubernetes Deployment Commands
```bash
# Create namespace and apply configurations
kubectl apply -f namespace.yaml
kubectl apply -f configmap.yaml

# Create secrets
kubectl create secret generic multios-secrets \
  --from-literal=database-password=your-secure-password \
  --namespace=multios-education

# Deploy database
kubectl apply -f postgres-deployment.yaml

# Deploy application
kubectl apply -f multios-deployment.yaml

# Check deployment status
kubectl get pods -n multios-education
kubectl get services -n multios-education
kubectl get ingress -n multios-education

# View logs
kubectl logs -f deployment/multios-portal -n multios-education

# Scale deployment
kubectl scale deployment multios-portal --replicas=5 -n multios-education
```

---

## Platform-Specific Guides

### AWS Deployment

#### CloudFormation Template
```yaml
AWSTemplateFormatVersion: '2010-09-09'
Description: 'MultiOS Educational Platform on AWS'

Parameters:
  InstitutionName:
    Type: String
    Description: Your institution name
  PartnershipTier:
    Type: String
    AllowedValues: [bronze, silver, gold, platinum]
    Default: silver
  KeyPairName:
    Type: AWS::EC2::KeyPair::KeyName
    Description: EC2 Key Pair for SSH access
  VpcId:
    Type: AWS::EC2::VPC::Id
    Description: VPC ID for deployment

Resources:
  # Security Group
  MultiosSecurityGroup:
    Type: AWS::EC2::SecurityGroup
    Properties:
      GroupDescription: Security group for MultiOS platform
      VpcId: !Ref VpcId
      SecurityGroupIngress:
        - IpProtocol: tcp
          FromPort: 80
          ToPort: 80
          CidrIp: 0.0.0.0/0
        - IpProtocol: tcp
          FromPort: 443
          ToPort: 443
          CidrIp: 0.0.0.0/0
        - IpProtocol: tcp
          FromPort: 22
          ToPort: 22
          CidrIp: !Ref AdminIp

  # RDS Instance
  MultiosDB:
    Type: AWS::RDS::DBInstance
    Properties:
      DBInstanceIdentifier: !Sub '${InstitutionName}-multios-db'
      Engine: postgres
      EngineVersion: '13.7'
      DBInstanceClass: !Ref DBInstanceClass
      AllocatedStorage: 100
      StorageType: gp2
      MasterUsername: multios_user
      MasterUserPassword: !Ref DBPassword
      DBName: multios_edu
      VPCSecurityGroups:
        - !Ref MultiosSecurityGroup
      MultiAZ: !Equals [!Ref Environment, 'production']
      BackupRetentionPeriod: 7
      DeletionProtection: true

  # Application Load Balancer
  MultiosALB:
    Type: AWS::ElasticLoadBalancingV2::LoadBalancer
    Properties:
      Name: !Sub '${InstitutionName}-multios-alb'
      Scheme: internet-facing
      Type: application
      SecurityGroups:
        - !Ref MultiosSecurityGroup
      Subnets:
        - !Ref PublicSubnet1
        - !Ref PublicSubnet2

  # Target Group
  MultiosTargetGroup:
    Type: AWS::ElasticLoadBalancingV2::TargetGroup
    Properties:
      Name: !Sub '${InstitutionName}-multios-tg'
      Port: 80
      Protocol: HTTP
      VpcId: !Ref VpcId
      TargetType: instance
      HealthCheckPath: /health
      HealthCheckIntervalSeconds: 30

  # Launch Configuration
  MultiosLaunchConfig:
    Type: AWS::AutoScaling::LaunchConfiguration
    Properties:
      ImageId: !Ref LatestAmiId
      InstanceType: !Ref InstanceType
      KeyName: !Ref KeyPairName
      SecurityGroups:
        - !Ref MultiosSecurityGroup
      UserData: !Base64 |
        #!/bin/bash
        yum update -y
        yum install -y docker postgresql postgresql-contrib
        systemctl enable docker
        systemctl start docker
        
        # Install MultiOS
        curl -sSL https://install.multios.edu | bash
        
        # Configure environment
        echo "DATABASE_HOST=<database-endpoint>" > /etc/multios/environment
        echo "INSTITUTION_NAME=${InstitutionName}" >> /etc/multios/environment
        echo "PARTNERSHIP_TIER=${PartnershipTier}" >> /etc/multios/environment
        
        systemctl enable multios
        systemctl start multios

  # Auto Scaling Group
  MultiosASG:
    Type: AWS::AutoScaling::AutoScalingGroup
    Properties:
      LaunchConfigurationName: !Ref MultiosLaunchConfig
      MinSize: 2
      MaxSize: 10
      DesiredCapacity: 3
      TargetGroupARNs:
        - !Ref MultiosTargetGroup
      VPCZoneIdentifier:
        - !Ref PrivateSubnet1
        - !Ref PrivateSubnet2
      HealthCheckType: ELB
      HealthCheckGracePeriod: 300

Outputs:
  LoadBalancerDNS:
    Description: DNS name of the load balancer
    Value: !GetAtt MultiosALB.DNSName
    Export:
      Name: !Sub '${AWS::StackName}-ALB-DNS'
```

### Azure Deployment

#### ARM Template
```json
{
  "$schema": "https://schema.management.azure.com/schemas/2019-04-01/deploymentTemplate.json#",
  "contentVersion": "1.0.0.0",
  "parameters": {
    "institutionName": {
      "type": "string",
      "metadata": {
        "description": "Name of the educational institution"
      }
    },
    "partnershipTier": {
      "type": "string",
      "allowedValues": ["bronze", "silver", "gold", "platinum"],
      "defaultValue": "silver"
    },
    "adminUsername": {
      "type": "string",
      "defaultValue": "multiosadmin"
    },
    "adminPassword": {
      "type": "securestring"
    },
    "vmSize": {
      "type": "string",
      "defaultValue": "Standard_D4s_v3"
    }
  },
  "variables": {
    "vnetName": "[concat(parameters('institutionName'), '-vnet')]",
    "subnetName": "default",
    "storageAccountName": "[concat('multios', uniqueString(parameters('institutionName')))]",
    "nicName": "[concat(parameters('institutionName'), '-nic')]",
    "vmName": "[concat(parameters('institutionName'), '-vm')]",
    "publicIPName": "[concat(parameters('institutionName'), '-pip')]",
    "loadBalancerName": "[concat(parameters('institutionName'), '-lb')]",
    "probeName": "[concat(parameters('institutionName'), '-probe')]",
    "backendPoolName": "[concat(parameters('institutionName'), '-backend-pool')]"
  },
  "resources": [
    {
      "type": "Microsoft.Storage/storageAccounts",
      "apiVersion": "2021-04-01",
      "name": "[variables('storageAccountName')]",
      "location": "[resourceGroup().location]",
      "sku": {
        "name": "Standard_LRS"
      },
      "kind": "StorageV2",
      "properties": {
        "accessTier": "Hot"
      }
    },
    {
      "type": "Microsoft.Network/publicIPAddresses",
      "apiVersion": "2021-02-01",
      "name": "[variables('publicIPName')]",
      "location": "[resourceGroup().location]",
      "properties": {
        "publicIPAllocationMethod": "Dynamic"
      }
    },
    {
      "type": "Microsoft.Network/loadBalancers",
      "apiVersion": "2021-02-01",
      "name": "[variables('loadBalancerName')]",
      "location": "[resourceGroup().location]",
      "properties": {
        "frontendIPConfigurations": [
          {
            "name": "LoadBalancerFrontEnd",
            "properties": {
              "publicIPAddress": {
                "id": "[resourceId('Microsoft.Network/publicIPAddresses', variables('publicIPName'))]"
              }
            }
          }
        ],
        "backendAddressPools": [
          {
            "name": "[variables('backendPoolName')]"
          }
        ],
        "probes": [
          {
            "name": "[variables('probeName')]",
            "properties": {
              "protocol": "Tcp",
              "port": 80,
              "intervalInSeconds": 15,
              "numberOfProbes": 2
            }
          }
        ],
        "loadBalancingRules": [
          {
            "name": "LoadBalancerRule",
            "properties": {
              "frontendIPConfiguration": {
                "id": "[concat(resourceId('Microsoft.Network/loadBalancers', variables('loadBalancerName')), '/frontendIPConfigurations/LoadBalancerFrontEnd')]"
              },
              "backendAddressPool": {
                "id": "[concat(resourceId('Microsoft.Network/loadBalancers', variables('loadBalancerName')), '/backendAddressPools/', variables('backendPoolName'))]"
              },
              "probe": {
                "id": "[concat(resourceId('Microsoft.Network/loadBalancers', variables('loadBalancerName')), '/probes/', variables('probeName'))]"
              },
              "protocol": "Tcp",
              "frontendPort": 80,
              "backendPort": 80,
              "enableFloatingIP": false
            }
          }
        ]
      }
    },
    {
      "type": "Microsoft.Network/networkInterfaces",
      "apiVersion": "2021-02-01",
      "name": "[variables('nicName')]",
      "location": "[resourceGroup().location]",
      "properties": {
        "ipConfigurations": [
          {
            "name": "ipconfig1",
            "properties": {
              "subnet": {
                "id": "[reference(resourceId('Microsoft.Network/virtualNetworks', variables('vnetName'))).subnets[0].id]"
              },
              "loadBalancerBackendAddressPools": [
                {
                  "id": "[concat(resourceId('Microsoft.Network/loadBalancers', variables('loadBalancerName')), '/backendAddressPools/', variables('backendPoolName'))]"
                }
              ],
              "publicIPAddress": {
                "id": "[resourceId('Microsoft.Network/publicIPAddresses', variables('publicIPName'))]"
              }
            }
          }
        ]
      },
      "dependsOn": [
        "[resourceId('Microsoft.Network/loadBalancers', variables('loadBalancerName'))]"
      ]
    },
    {
      "type": "Microsoft.Compute/virtualMachines",
      "apiVersion": "2021-03-01",
      "name": "[variables('vmName')]",
      "location": "[resourceGroup().location]",
      "properties": {
        "hardwareProfile": {
          "vmSize": "[parameters('vmSize')]"
        },
        "osProfile": {
          "computerName": "[variables('vmName')]",
          "adminUsername": "[parameters('adminUsername')]",
          "adminPassword": "[parameters('adminPassword')]"
        },
        "storageProfile": {
          "imageReference": {
            "publisher": "Canonical",
            "offer": "UbuntuServer",
            "sku": "18.04-LTS",
            "version": "latest"
          },
          "osDisk": {
            "name": "[concat(variables('vmName'), '-osdisk')]",
            "createOption": "FromImage"
          }
        },
        "networkProfile": {
          "networkInterfaces": [
            {
              "id": "[resourceId('Microsoft.Network/networkInterfaces', variables('nicName'))]"
            }
          ]
        }
      },
      "dependsOn": [
        "[resourceId('Microsoft.Network/networkInterfaces', variables('nicName'))]"
      ]
    }
  ]
}
```

---

## Configuration Management

### Environment Configuration

#### Production Environment File
```bash
# /etc/multios/environment
# MultiOS Educational Platform Configuration

# Institution Information
INSTITUTION_NAME="Your Institution Name"
PARTNERSHIP_TIER="silver"
ADMIN_EMAIL="admin@institution.edu"

# Database Configuration
DATABASE_HOST="localhost"
DATABASE_PORT="5432"
DATABASE_NAME="multios_edu"
DATABASE_USER="multios_user"
DATABASE_PASSWORD="secure_password_here"

# Redis Configuration
REDIS_HOST="localhost"
REDIS_PORT="6379"
REDIS_PASSWORD=""

# Web Server Configuration
WEB_SERVER_PORT="80"
SSL_ENABLED="true"
SSL_CERT_PATH="/etc/ssl/certs/multios.crt"
SSL_KEY_PATH="/etc/ssl/private/multios.key"

# Email Configuration
SMTP_HOST="smtp.institution.edu"
SMTP_PORT="587"
SMTP_USER="noreply@institution.edu"
SMTP_PASSWORD="email_password"
SMTP_USE_TLS="true"

# Security Configuration
SESSION_SECRET="generate_random_secret_here"
JWT_SECRET="generate_jwt_secret_here"
ENCRYPTION_KEY="generate_encryption_key_here"

# Logging Configuration
LOG_LEVEL="info"
LOG_FILE="/var/log/multios/application.log"
LOG_MAX_SIZE="100M"
LOG_MAX_FILES="10"

# Performance Configuration
MAX_CONNECTIONS="1000"
WORKER_PROCESSES="4"
CACHE_TTL="3600"

# Integration Configuration
LMS_INTEGRATION_ENABLED="true"
LDAP_ENABLED="true"
SAML_ENABLED="false"
API_RATE_LIMIT="1000"

# Backup Configuration
BACKUP_ENABLED="true"
BACKUP_SCHEDULE="0 2 * * *"
BACKUP_RETENTION_DAYS="30"
S3_BACKUP_BUCKET="multios-backups"

# Monitoring Configuration
MONITORING_ENABLED="true"
METRICS_ENDPOINT="/metrics"
HEALTH_CHECK_ENDPOINT="/health"

# Development Configuration (disable in production)
DEBUG_MODE="false"
DEV_TOOLS="false"
API_DOCS_ENABLED="false"
```

#### Docker Environment Configuration
```bash
# docker-compose.yml environment section
environment:
  # Required environment variables
  - INSTITUTION_NAME=${INSTITUTION_NAME}
  - PARTNERSHIP_TIER=${PARTNERSHIP_TIER}
  - DATABASE_HOST=postgres
  - DATABASE_NAME=multios_edu
  - DATABASE_USER=multios_user
  - DATABASE_PASSWORD=${DB_PASSWORD}
  - REDIS_HOST=redis
  
  # Optional configuration
  - SMTP_HOST=${SMTP_HOST}
  - SMTP_USER=${SMTP_USER}
  - SMTP_PASSWORD=${SMTP_PASSWORD}
  - SESSION_SECRET=${SESSION_SECRET}
  - JWT_SECRET=${JWT_SECRET}
  - ENCRYPTION_KEY=${ENCRYPTION_KEY}
  - LOG_LEVEL=info
  - SSL_ENABLED=true
  
  # Feature flags
  - LMS_INTEGRATION_ENABLED=true
  - LDAP_ENABLED=true
  - BACKUP_ENABLED=true
  - MONITORING_ENABLED=true
```

### Configuration Validation Script
```bash
#!/bin/bash
# Configuration validation script

validate_environment() {
    echo "Validating MultiOS configuration..."
    
    # Check required environment variables
    required_vars=(
        "INSTITUTION_NAME"
        "PARTNERSHIP_TIER"
        "DATABASE_HOST"
        "DATABASE_NAME"
        "DATABASE_USER"
        "DATABASE_PASSWORD"
    )
    
    for var in "${required_vars[@]}"; do
        if [ -z "${!var}" ]; then
            echo "Error: Required environment variable $var is not set"
            exit 1
        fi
    done
    
    # Validate database connection
    echo "Testing database connection..."
    PGPASSWORD="$DATABASE_PASSWORD" psql -h "$DATABASE_HOST" -U "$DATABASE_USER" -d "$DATABASE_NAME" -c "SELECT 1;" > /dev/null
    if [ $? -ne 0 ]; then
        echo "Error: Cannot connect to database"
        exit 1
    fi
    
    # Validate Redis connection
    echo "Testing Redis connection..."
    redis-cli -h "$REDIS_HOST" ping > /dev/null
    if [ $? -ne 0 ]; then
        echo "Error: Cannot connect to Redis"
        exit 1
    fi
    
    # Check SSL certificates if SSL is enabled
    if [ "$SSL_ENABLED" = "true" ]; then
        echo "Validating SSL certificates..."
        if [ ! -f "$SSL_CERT_PATH" ] || [ ! -f "$SSL_KEY_PATH" ]; then
            echo "Error: SSL certificate or key file not found"
            exit 1
        fi
    fi
    
    echo "Configuration validation completed successfully"
}

# Run validation
validate_environment
```

---

## Integration Setup

### LMS Integration Examples

#### Canvas Integration
```python
# canvas_integration.py
import requests
from requests.auth import HTTPBasicAuth
import json

class CanvasIntegration:
    def __init__(self, base_url, access_token):
        self.base_url = base_url
        self.headers = {
            'Authorization': f'Bearer {access_token}',
            'Content-Type': 'application/json'
        }
    
    def get_courses(self):
        """Fetch courses from Canvas"""
        response = requests.get(
            f'{self.base_url}/api/v1/courses',
            headers=self.headers
        )
        return response.json()
    
    def enroll_students(self, course_id, students):
        """Enroll students in MultiOS course"""
        enrollment_data = {
            'enrollment': {
                'type': 'StudentEnrollment',
                'course_id': course_id,
                'user_id': students['canvas_user_id'],
                'enrollment_state': 'active'
            }
        }
        
        response = requests.post(
            f'{self.base_url}/api/v1/courses/{course_id}/enrollments',
            headers=self.headers,
            json=enrollment_data
        )
        return response.json()
    
    def sync_grades(self, course_id, assignments):
        """Sync grades back to Canvas"""
        for assignment in assignments:
            grade_data = {
                'submission': {
                    'posted_grade': assignment['grade']
                }
            }
            
            response = requests.put(
                f'{self.base_url}/api/v1/courses/{course_id}/assignments/{assignment["id"]}/submissions/{assignment["user_id"]}',
                headers=self.headers,
                json=grade_data
            )
            return response.json()

# Usage example
canvas = CanvasIntegration('https://your-institution.instructure.com', 'your-access-token')
courses = canvas.get_courses()
```

#### Moodle Integration
```php
<?php
// moodle_integration.php
class MoodleIntegration {
    private $ws_url;
    private $token;
    
    public function __construct($ws_url, $token) {
        $this->ws_url = $ws_url;
        $this->token = $token;
    }
    
    public function get_users($criteria) {
        $params = [
            'wsfunction' => 'core_user_get_users',
            'wstoken' => $this->token,
            'criteria' => $criteria
        ];
        
        $response = $this->make_request($params);
        return json_decode($response, true);
    }
    
    public function enroll_users($course_id, $user_ids) {
        $enrollments = [];
        foreach ($user_ids as $user_id) {
            $enrollments[] = [
                'roleid' => 5, // Student role ID
                'userid' => $user_id,
                'courseid' => $course_id
            ];
        }
        
        $params = [
            'wsfunction' => 'enrol_manual_enrol_users',
            'wstoken' => $this->token,
            'enrolments' => $enrollments
        ];
        
        $response = $this->make_request($params);
        return json_decode($response, true);
    }
    
    private function make_request($params) {
        $url = $this->ws_url . '/webservice/rest/server.php';
        
        $query = http_build_query($params);
        $ch = curl_init();
        curl_setopt($ch, CURLOPT_URL, $url);
        curl_setopt($ch, CURLOPT_POST, true);
        curl_setopt($ch, CURLOPT_POSTFIELDS, $query);
        curl_setopt($ch, CURLOPT_RETURNTRANSFER, true);
        
        $response = curl_exec($ch);
        curl_close($ch);
        
        return $response;
    }
}

// Usage example
$moodle = new MoodleIntegration('https://your-moodle-site.com', 'your-token');
$users = $moodle->get_users([['key' => 'email', 'value' => '@institution.edu']]);
```

### LDAP/Active Directory Integration

#### LDAP Configuration
```yaml
# /etc/multios/ldap.yml
ldap:
  enabled: true
  server: ldap://ldap.institution.edu
  port: 389
  bind_dn: cn=multios,ou=service_accounts,dc=institution,dc=edu
  bind_password: secure_password_here
  search_base: ou=people,dc=institution,dc=edu
  search_filter: "(uid={username})"
  user_attribute: uid
  group_attribute: memberOf
  
  # Group mapping
  groups:
    faculty: "cn=faculty,ou=groups,dc=institution,dc=edu"
    staff: "cn=staff,ou=groups,dc=institution,dc=edu"
    students: "cn=students,ou=groups,dc=institution,dc=edu"
    admin: "cn=administrators,ou=groups,dc=institution,dc=edu"
  
  # Field mapping
  mapping:
    username: uid
    email: mail
    first_name: givenName
    last_name: sn
    display_name: displayName
    
  # SSL/TLS
  use_ssl: true
  verify_cert: true
  ca_cert_path: /etc/ssl/certs/institution-ca.pem
```

#### LDAP Authentication Script
```python
# ldap_auth.py
import ldap3
from ldap3 import Server, Connection, ALL, SASL, DIGEST_MD5

class LDAPAuthentication:
    def __init__(self, server_uri, bind_dn, bind_password, search_base):
        self.server = Server(server_uri, get_info=ALL)
        self.bind_dn = bind_dn
        self.bind_password = bind_password
        self.search_base = search_base
        self.conn = None
    
    def connect(self):
        """Establish LDAP connection"""
        self.conn = Connection(
            self.server,
            user=self.bind_dn,
            password=self.bind_password,
            auto_bind=True
        )
        return self.conn.bind()
    
    def authenticate(self, username, password):
        """Authenticate user against LDAP"""
        # Search for user
        search_filter = f"(uid={username})"
        self.conn.search(
            search_base=self.search_base,
            search_filter=search_filter,
            attributes=['uid', 'mail', 'givenName', 'sn']
        )
        
        if not self.conn.entries:
            return False, "User not found"
        
        user_entry = self.conn.entries[0]
        user_dn = user_entry.entry_dn
        
        # Attempt bind with user credentials
        try:
            user_conn = Connection(
                self.server,
                user=user_dn,
                password=password,
                auto_bind=True
            )
            return True, {
                'username': str(user_entry.uid),
                'email': str(user_entry.mail),
                'first_name': str(user_entry.givenName),
                'last_name': str(user_entry.sn)
            }
        except ldap3.core.exceptions.LDAPBindError:
            return False, "Invalid credentials"
    
    def get_user_groups(self, username):
        """Get user's group memberships"""
        search_filter = f"(uid={username})"
        self.conn.search(
            search_base=self.search_base,
            search_filter=search_filter,
            attributes=['memberOf']
        )
        
        if self.conn.entries:
            return str(self.conn.entries[0].memberOf).split(',')
        return []
    
    def disconnect(self):
        """Close LDAP connection"""
        if self.conn:
            self.conn.unbind()

# Usage example
ldap_auth = LDAPAuthentication(
    'ldap://ldap.institution.edu',
    'cn=multios,ou=service_accounts,dc=institution,dc=edu',
    'secure_password',
    'ou=people,dc=institution,dc=edu'
)

ldap_auth.connect()
success, user_info = ldap_auth.authenticate('student123', 'password')
if success:
    print(f"Authenticated: {user_info}")
    groups = ldap_auth.get_user_groups('student123')
    print(f"Groups: {groups}")
```

---

## Troubleshooting Guide

### Common Installation Issues

#### Database Connection Errors
```bash
# Check PostgreSQL service status
sudo systemctl status postgresql

# Check PostgreSQL configuration
sudo -u postgres psql -c "SHOW config_file;"

# Test database connection
PGPASSWORD=your_password psql -h localhost -U multios_user -d multios_edu -c "SELECT 1;"

# Check PostgreSQL logs
sudo tail -f /var/log/postgresql/postgresql-*.log

# Common fixes
sudo -u postgres createdb multios_edu
sudo -u postgres createuser multios_user
sudo -u postgres psql -c "ALTER USER multios_user PASSWORD 'password';"
sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE multios_edu TO multios_user;"
```

#### Web Server Issues
```bash
# Check nginx configuration
sudo nginx -t

# Check nginx status
sudo systemctl status nginx

# View nginx error logs
sudo tail -f /var/log/nginx/error.log

# Check if port is already in use
sudo netstat -tlnp | grep :80

# Restart services
sudo systemctl restart nginx
sudo systemctl restart multios
```

#### Permission Issues
```bash
# Fix file permissions
sudo chown -R multios:multios /opt/multios
sudo chown -R multios:multios /var/log/multios
sudo chown -R multios:multios /etc/multios

# Set correct permissions
sudo chmod -R 755 /opt/multios
sudo chmod 600 /etc/multios/environment
sudo chmod 644 /var/log/multios/*.log
```

### Performance Troubleshooting

#### System Resource Monitoring
```bash
# Check CPU and memory usage
top
htop
free -h
df -h

# Check disk I/O
iostat -x 1
iotop

# Check network connections
ss -tuln
netstat -i

# Check MultiOS process
ps aux | grep multios
systemctl status multios
```

#### Database Performance
```bash
# Check PostgreSQL performance
sudo -u postgres psql -d multios_edu -c "
    SELECT query, calls, total_time, mean_time 
    FROM pg_stat_statements 
    ORDER BY total_time DESC 
    LIMIT 10;
"

# Check database connections
sudo -u postgres psql -c "
    SELECT count(*) as active_connections,
           max_connections,
           100.0 * count(*) / max_connections as utilization_percent
    FROM pg_stat_activity, 
         (SELECT setting::int as max_connections FROM pg_settings WHERE name = 'max_connections') m;
"

# Check slow queries
sudo -u postgres psql -d multios_edu -c "
    SELECT query, mean_time, calls 
    FROM pg_stat_statements 
    WHERE mean_time > 1000 
    ORDER BY mean_time DESC 
    LIMIT 10;
"
```

#### Application Performance
```bash
# Check MultiOS logs for errors
tail -f /var/log/multios/application.log | grep ERROR

# Monitor API response times
curl -w "@curl-format.txt" -o /dev/null -s http://localhost:80/health

# Check memory usage by MultiOS process
pmap -x $(pgrep multios)

# Check file descriptors
lsof -p $(pgrep multios) | wc -l
```

### Log Analysis and Monitoring

#### Log Locations
```bash
# MultiOS application logs
/var/log/multios/application.log
/var/log/multios/access.log
/var/log/multios/error.log

# System logs
/var/log/syslog
/var/log/messages

# Web server logs
/var/log/nginx/access.log
/var/log/nginx/error.log

# Database logs
/var/log/postgresql/postgresql-*.log
```

#### Log Analysis Commands
```bash
# Search for errors
grep -i error /var/log/multios/application.log | tail -20

# Check for connection issues
grep -i "connection" /var/log/multios/application.log | tail -10

# Monitor real-time logs
tail -f /var/log/multios/application.log

# Search by date range
grep "2023-11-01" /var/log/multios/application.log

# Count error types
grep -i error /var/log/multios/application.log | awk '{print $5}' | sort | uniq -c
```

### Health Checks and Monitoring

#### Health Check Script
```bash
#!/bin/bash
# health_check.sh

# Check MultiOS service
check_multios() {
    if systemctl is-active --quiet multios; then
        echo "✓ MultiOS service is running"
        return 0
    else
        echo "✗ MultiOS service is not running"
        return 1
    fi
}

# Check database connection
check_database() {
    if PGPASSWORD=$DB_PASSWORD psql -h $DB_HOST -U $DB_USER -d $DB_NAME -c "SELECT 1;" > /dev/null 2>&1; then
        echo "✓ Database connection is working"
        return 0
    else
        echo "✗ Database connection failed"
        return 1
    fi
}

# Check web server
check_web() {
    if curl -f -s http://localhost:80/health > /dev/null; then
        echo "✓ Web server is responding"
        return 0
    else
        echo "✗ Web server is not responding"
        return 1
    fi
}

# Check disk space
check_disk() {
    DISK_USAGE=$(df / | awk 'NR==2 {print $5}' | sed 's/%//')
    if [ $DISK_USAGE -lt 80 ]; then
        echo "✓ Disk usage is acceptable ($DISK_USAGE%)"
        return 0
    else
        echo "✗ Disk usage is high ($DISK_USAGE%)"
        return 1
    fi
}

# Check memory usage
check_memory() {
    MEM_USAGE=$(free | awk 'NR==2{printf "%.0f", $3*100/$2}')
    if [ $MEM_USAGE -lt 80 ]; then
        echo "✓ Memory usage is acceptable ($MEM_USAGE%)"
        return 0
    else
        echo "✗ Memory usage is high ($MEM_USAGE%)"
        return 1
    fi
}

# Main health check
echo "Running MultiOS health check..."
check_multios
check_database
check_web
check_disk
check_memory
echo "Health check completed."
```

---

## Support and Contact Information

**Deployment Support:**
- Email: deployment@multios.edu
- Phone: +1 (555) MULTI-OS
- Portal: https://support.multios.edu

**Technical Documentation:**
- Installation Guides: https://docs.multios.edu/installation
- Configuration Reference: https://docs.multios.edu/configuration
- API Documentation: https://api.multios.edu/docs

**Community Support:**
- Forum: https://community.multios.edu
- GitHub: https://github.com/multios
- Slack: https://multios.slack.com

**24/7 Emergency Support (Platinum Partners):**
- Emergency Hotline: +1 (555) MULTI-911
- Emergency Email: emergency@multios.edu

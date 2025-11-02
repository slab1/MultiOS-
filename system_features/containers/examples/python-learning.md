# Python Learning Container Example

This example demonstrates how to create and run a Python learning container for educational purposes.

## Prerequisites

- MultiOS Container System installed
- Root privileges (sudo)
- Docker-compatible understanding (helpful but not required)

## Quick Start Example

### Step 1: List Available Templates

First, check what educational templates are available:

```bash
multios-container template list
```

You should see output similar to:
```
Available Templates:
  - python-learning: Python 3.9+ with common packages
  - nodejs-learning: Node.js 16+ with npm and tools  
  - java-learning: OpenJDK 11 with Maven/Gradle
  - cpp-learning: GCC 9+ with build tools
  - web-learning: Full web development stack
  - database-learning: Multiple database servers
  - networking-learning: Network analysis tools
```

### Step 2: Create a Python Learning Container

Create a new Python learning environment:

```bash
# Create container with default settings
multios-container create --template python-learning python-course

# Or with custom resource limits
multios-container create \
  --template python-learning \
  --name python-course \
  --cpu-limit 2 \
  --memory-limit 2G \
  --disk-limit 10G \
  python-course
```

### Step 3: Start the Container

```bash
multios-container start python-course
```

Check the container status:
```bash
multios-container ps
```

### Step 4: Explore the Python Environment

Execute Python commands in the container:

```bash
# Check Python version
multios-container exec python-course python3 --version

# Start Python interactive shell
multios-container exec python-course python3

# Run a Python script
multios-container exec python-course python3 -c "print('Hello from MultiOS Container!')"
```

### Step 5: Install Additional Packages

```bash
# Install a Python package
multios-container exec python-course pip3 install requests

# Install multiple packages
multios-container exec python-course pip3 install numpy pandas matplotlib

# Verify installation
multios-container exec python-course python3 -c "import requests; print('requests installed successfully')"
```

## Detailed Example: Data Science Workflow

Let's create a comprehensive data science learning environment.

### Setup the Container

```bash
# Create container with sufficient resources for data science
multios-container create \
  --template python-learning \
  --name data-science-lab \
  --cpu-limit 4 \
  --memory-limit 4G \
  --disk-limit 20G \
  python-learning

# Start the container
multios-container start data-science-lab
```

### Install Data Science Libraries

```bash
# Install Jupyter Notebook
multios-container exec data-science-lab pip3 install jupyter notebook

# Install data science libraries
multios-container exec data-science-lab pip3 install \
    numpy \
    pandas \
    matplotlib \
    seaborn \
    scikit-learn \
    plotly \
    bokeh \
    scipy

# Install database connectivity
multios-container exec data-science-lab pip3 install \
    psycopg2-binary \
    pymongo \
    sqlalchemy
```

### Create a Sample Data Science Project

```bash
# Create project directory
multios-container exec data-science-lab mkdir -p /workspace/data-science

# Create a sample analysis script
multios-container exec data-science-lab bash -c 'cat > /workspace/data-science/sample_analysis.py << EOF
import numpy as np
import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
from sklearn.datasets import load_iris
from sklearn.model_selection import train_test_split
from sklearn.ensemble import RandomForestClassifier
from sklearn.metrics import accuracy_score, classification_report

def load_and_explore_data():
    """Load the Iris dataset and perform basic exploration."""
    print("Loading Iris dataset...")
    iris = load_iris()
    df = pd.DataFrame(iris.data, columns=iris.feature_names)
    df['target'] = iris.target
    df['species'] = df['target'].map({0: 'setosa', 1: 'versicolor', 2: 'virginica'})
    
    print(f"Dataset shape: {df.shape}")
    print(f"Target distribution:\n{df['species'].value_counts()}")
    print(f"\nFirst few rows:\n{df.head()}")
    print(f"\nDataset info:")
    print(df.info())
    
    return df

def create_visualizations(df):
    """Create various visualizations of the dataset."""
    print("\nCreating visualizations...")
    
    # Set up the plotting style
    plt.style.use('seaborn-v0_8')
    fig, axes = plt.subplots(2, 2, figsize=(15, 12))
    
    # 1. Pairplot showing relationships
    sns.pairplot(df, hue='species', markers=['o', 's', '^'], height=3)
    plt.savefig('/workspace/data-science/pairplot.png', dpi=300, bbox_inches='tight')
    
    # 2. Correlation heatmap
    plt.figure(figsize=(10, 8))
    correlation_matrix = df.iloc[:, :-2].corr()  # Exclude target and species columns
    sns.heatmap(correlation_matrix, annot=True, cmap='coolwarm', center=0)
    plt.title('Feature Correlation Matrix')
    plt.savefig('/workspace/data-science/correlation.png', dpi=300, bbox_inches='tight')
    
    # 3. Distribution plots
    fig, axes = plt.subplots(2, 2, figsize=(15, 10))
    features = ['sepal length (cm)', 'sepal width (cm)', 
               'petal length (cm)', 'petal width (cm)']
    
    for i, feature in enumerate(features):
        row, col = i // 2, i % 2
        for species in df['species'].unique():
            data = df[df['species'] == species][feature]
            axes[row, col].hist(data, alpha=0.6, label=species, bins=15)
        axes[row, col].set_title(f'Distribution of {feature}')
        axes[row, col].legend()
    
    plt.tight_layout()
    plt.savefig('/workspace/data-science/distributions.png', dpi=300, bbox_inches='tight')
    
    plt.close('all')  # Close all figures to free memory
    print("Visualizations saved to /workspace/data-science/")

def train_model(df):
    """Train a Random Forest classifier on the dataset."""
    print("\nTraining Random Forest classifier...")
    
    # Prepare features and target
    X = df.iloc[:, :-2]  # Features (exclude target and species)
    y = df['target']
    
    # Split data
    X_train, X_test, y_train, y_test = train_test_split(
        X, y, test_size=0.3, random_state=42, stratify=y
    )
    
    # Train model
    model = RandomForestClassifier(n_estimators=100, random_state=42)
    model.fit(X_train, y_train)
    
    # Make predictions
    y_pred = model.predict(X_test)
    
    # Evaluate model
    accuracy = accuracy_score(y_test, y_pred)
    print(f"Model accuracy: {accuracy:.3f}")
    print(f"\nClassification Report:\n{classification_report(y_test, y_pred)}")
    
    # Feature importance
    feature_importance = pd.DataFrame({
        'feature': X.columns,
        'importance': model.feature_importances_
    }).sort_values('importance', ascending=False)
    
    print(f"\nFeature Importance:\n{feature_importance}")
    
    return model, accuracy

def create_sample_data():
    """Create a sample dataset for practice."""
    print("\nCreating sample dataset...")
    
    np.random.seed(42)
    n_samples = 1000
    
    # Generate synthetic data
    age = np.random.normal(35, 10, n_samples)
    income = np.random.normal(50000, 15000, n_samples)
    experience = np.random.normal(10, 5, n_samples)
    
    # Create a binary target variable
    # People with higher income and experience are more likely to be "employed"
    score = (income - 50000) / 15000 + (experience - 10) / 5
    employed = (score + np.random.normal(0, 0.5, n_samples)) > 0
    
    sample_df = pd.DataFrame({
        'age': np.clip(age, 18, 65),
        'income': np.clip(income, 20000, 150000),
        'experience': np.clip(experience, 0, 40),
        'employed': employed
    })
    
    print(f"Created sample dataset with {len(sample_df)} records")
    print(f"Employment rate: {sample_df['employed'].mean():.2%}")
    
    return sample_df

def main():
    """Main function to run the complete analysis."""
    print("=" * 50)
    print("Data Science Learning Example")
    print("MultiOS Container System")
    print("=" * 50)
    
    # 1. Load and explore real dataset
    df = load_and_explore_data()
    
    # 2. Create visualizations
    create_visualizations(df)
    
    # 3. Train a machine learning model
    model, accuracy = train_model(df)
    
    # 4. Create sample dataset
    sample_df = create_sample_data()
    
    # Save sample dataset
    sample_df.to_csv('/workspace/data-science/sample_dataset.csv', index=False)
    print(f"\nSample dataset saved to /workspace/data-science/sample_dataset.csv")
    
    print("\n" + "=" * 50)
    print("Analysis complete! Check the generated files:")
    print("- pairplot.png: Pairwise feature relationships")
    print("- correlation.png: Feature correlation heatmap")
    print("- distributions.png: Feature distributions by species")
    print("- sample_dataset.csv: Synthetic dataset for practice")
    print("=" * 50)

if __name__ == "__main__":
    main()
EOF'
```

### Run the Analysis

```bash
# Execute the data science script
multios-container exec data-science-lab python3 /workspace/data-science/sample_analysis.py
```

### Start Jupyter Notebook

```bash
# Start Jupyter Notebook (runs in background)
multios-container exec -d data-science-lab jupyter notebook --ip=0.0.0.0 --port=8888 --no-browser --allow-root

# Check if Jupyter is running
multios-container exec data-science-lab ps aux | grep jupyter

# Access Jupyter (if networking is configured)
# Open browser to http://localhost:8888
```

### View Results

```bash
# List generated files
multios-container exec data-science-lab ls -la /workspace/data-science/

# View a generated CSV file
multios-container exec data-science-lab head /workspace/data-science/sample_dataset.csv

# Copy files back to host (if needed)
multios-container exec data-science-lab tar -czf results.tar.gz -C /workspace data-science/
multios-container cp data-science-lab:/workspace/results.tar.gz ./results.tar.gz
```

## Web Development Example

Let's create a simple web application:

```bash
# Create web development container
multios-container create \
  --template web-learning \
  --name web-dev-lab \
  --port 8080:8080 \
  web-learning

multios-container start web-dev-lab

# Install Flask web framework
multios-container exec web-dev-lab pip3 install flask flask-sqlalchemy

# Create a simple web application
multios-container exec web-dev-lab bash -c 'cat > /workspace/app.py << EOF
from flask import Flask, render_template_string
import sqlite3
from datetime import datetime

app = Flask(__name__)

# Simple HTML template
HTML_TEMPLATE = """
<!DOCTYPE html>
<html>
<head>
    <title>MultiOS Container Demo</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        .container { max-width: 800px; margin: 0 auto; }
        .header { background-color: #007acc; color: white; padding: 20px; border-radius: 5px; }
        .content { margin-top: 20px; padding: 20px; border: 1px solid #ddd; border-radius: 5px; }
        .footer { margin-top: 20px; text-align: center; color: #666; }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🌟 MultiOS Container Demo</h1>
            <p>Python Web Application running in a Container</p>
        </div>
        <div class="content">
            <h2>Container Information</h2>
            <ul>
                <li><strong>Container Name:</strong> {{ container_name }}</li>
                <li><strong>Host:</strong> {{ hostname }}</li>
                <li><strong>Current Time:</strong> {{ current_time }}</li>
                <li><strong>Python Version:</strong> {{ python_version }}</li>
            </ul>
            
            <h2>System Statistics</h2>
            <ul>
                <li><strong>Disk Usage:</strong> {{ disk_usage }}</li>
                <li><strong>Memory Usage:</strong> {{ memory_info }}</li>
                <li><strong>CPU Info:</strong> {{ cpu_info }}</li>
            </ul>
        </div>
        <div class="footer">
            <p>Powered by MultiOS Container System</p>
        </div>
    </div>
</body>
</html>
"""

@app.route('/')
def index():
    import os
    import platform
    import psutil
    
    # Get system information
    template_data = {
        'container_name': os.environ.get('HOSTNAME', 'Unknown'),
        'hostname': platform.node(),
        'current_time': datetime.now().strftime('%Y-%m-%d %H:%M:%S'),
        'python_version': platform.python_version(),
        'disk_usage': f"{psutil.disk_usage('/').used // (1024**3)}GB / {psutil.disk_usage('/').total // (1024**3)}GB",
        'memory_info': f"{psutil.virtual_memory().used // (1024**3)}GB / {psutil.virtual_memory().total // (1024**3)}GB",
        'cpu_info': f"{psutil.cpu_count()} cores @ {psutil.cpu_freq().current:.0f}MHz"
    }
    
    return render_template_string(HTML_TEMPLATE, **template_data)

@app.route('/health')
def health():
    return {
        'status': 'healthy',
        'timestamp': datetime.now().isoformat(),
        'container': os.environ.get('HOSTNAME', 'Unknown')
    }

if __name__ == '__main__':
    print("Starting Flask application...")
    print("Access the application at http://localhost:8080")
    app.run(host='0.0.0.0', port=8080, debug=True)
EOF'

# Install required system monitoring library
multios-container exec web-dev-lab pip3 install psutil

# Run the web application
multios-container exec -d web-dev-lab python3 /workspace/app.py
```

## Cleanup

When you're done with the containers:

```bash
# Stop containers
multios-container stop python-course
multios-container stop data-science-lab
multios-container stop web-dev-lab

# Remove containers
multios-container delete python-course
multios-container delete data-science-lab
multios-container delete web-dev-lab

# List remaining containers
multios-container ps --all
```

## Advanced Usage

### Custom Resource Limits

```bash
# Create container with specific resource constraints
multios-container create \
  --template python-learning \
  --name constrained-python \
  --cpu-limit 0.5 \
  --memory-limit 512M \
  --disk-limit 5G \
  --cpu-shares 256 \
  python-learning
```

### Volume Mounting

```bash
# Mount host directory to container
multios-container create \
  --template python-learning \
  --name dev-env \
  --volume /host/projects:/workspace \
  --volume /host/config:/root/.config \
  python-learning
```

### Network Isolation

```bash
# Create isolated network
multios-container network create isolated-net --subnet 10.0.1.0/24

# Create container in isolated network
multios-container create \
  --template python-learning \
  --name isolated-python \
  --network isolated-net \
  python-learning
```

### Security Hardening

```bash
# Create container with security restrictions
multios-container create \
  --template python-learning \
  --name secure-python \
  --read-only \
  --cap-drop ALL \
  --cap-add CHOWN,SETGID,SETUID \
  --security-opt apparmor=default \
  python-learning
```

## Troubleshooting

### Container Won't Start

```bash
# Check container logs
multios-container logs python-course

# Inspect container configuration
multios-container inspect python-course

# Verify resources are available
multios-container stats
```

### Network Connectivity Issues

```bash
# Check network configuration
multios-container network list

# Test connectivity from container
multios-container exec python-course ping 8.8.8.8
```

### Performance Issues

```bash
# Monitor container resource usage
multios-container stats python-course

# Update resource limits
multios-container update python-course --memory-limit 4G
```

This example demonstrates the full capabilities of the MultiOS Container System for Python education and development. You can adapt these patterns for other programming languages and use cases using the available templates.

---

**Next Steps:**
- Explore other educational templates (Java, C++, Node.js, etc.)
- Try the orchestration features for multi-container applications
- Experiment with custom resource limits and security configurations
- Use the container template system to create custom learning environments
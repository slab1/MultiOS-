"""
Machine Learning Algorithms from Scratch for Scientific Computing Education
=========================================================================

This module provides educational implementations of machine learning algorithms:
- Supervised learning algorithms (linear regression, decision trees, SVMs, neural networks)
- Unsupervised learning algorithms (clustering, PCA, association rules)
- Evaluation metrics and cross-validation
- Feature engineering and preprocessing

Author: Scientific Computing Education Team
"""

import numpy as np
import matplotlib.pyplot as plt
from typing import Dict, List, Tuple, Optional, Callable, Union
from abc import ABC, abstractmethod
from dataclasses import dataclass
import math


@dataclass
class ModelMetrics:
    """Container for model evaluation metrics."""
    accuracy: float
    precision: float
    recall: float
    f1_score: float
    auc_roc: float
    confusion_matrix: np.ndarray


class DataPreprocessing:
    """Data preprocessing utilities for machine learning."""
    
    @staticmethod
    def standardize(X: np.ndarray) -> Tuple[np.ndarray, np.ndarray, np.ndarray]:
        """
        Standardize features to zero mean and unit variance.
        
        Args:
            X: Input feature matrix (samples x features)
            
        Returns:
            Tuple of (standardized_X, mean, std)
        """
        mean = np.mean(X, axis=0)
        std = np.std(X, axis=0)
        std = np.where(std == 0, 1, std)  # Avoid division by zero
        
        X_standardized = (X - mean) / std
        return X_standardized, mean, std
    
    @staticmethod
    def min_max_scale(X: np.ndarray, feature_range: Tuple[float, float] = (0, 1)) -> Tuple[np.ndarray, float, float]:
        """
        Scale features to specified range.
        
        Args:
            X: Input feature matrix
            feature_range: Target range (min, max)
            
        Returns:
            Tuple of (scaled_X, min_val, max_val)
        """
        min_val = np.min(X, axis=0)
        max_val = np.max(X, axis=0)
        range_val = max_val - min_val
        range_val = np.where(range_val == 0, 1, range_val)  # Avoid division by zero
        
        min_scale, max_scale = feature_range
        X_scaled = min_scale + (X - min_val) * (max_scale - min_scale) / range_val
        
        return X_scaled, min_val, max_val
    
    @staticmethod
    def one_hot_encode(y: np.ndarray) -> np.ndarray:
        """
        Convert class labels to one-hot encoding.
        
        Args:
            y: Class labels (1D array)
            
        Returns:
            One-hot encoded matrix
        """
        unique_classes = np.unique(y)
        n_classes = len(unique_classes)
        class_to_idx = {cls: idx for idx, cls in enumerate(unique_classes)}
        
        y_encoded = np.zeros((len(y), n_classes))
        for i, label in enumerate(y):
            y_encoded[i, class_to_idx[label]] = 1
            
        return y_encoded
    
    @staticmethod
    def train_test_split(X: np.ndarray, y: np.ndarray, test_size: float = 0.2, 
                        random_state: Optional[int] = None) -> Tuple[np.ndarray, np.ndarray, np.ndarray, np.ndarray]:
        """
        Split data into training and testing sets.
        
        Args:
            X: Feature matrix
            y: Target vector
            test_size: Fraction of data for testing
            random_state: Random seed
            
        Returns:
            Tuple of (X_train, X_test, y_train, y_test)
        """
        if random_state is not None:
            np.random.seed(random_state)
        
        n_samples = len(X)
        n_test = int(n_samples * test_size)
        
        indices = np.random.permutation(n_samples)
        test_indices = indices[:n_test]
        train_indices = indices[n_test:]
        
        return (X[train_indices], X[test_indices], y[train_indices], y[test_indices])


class BaseModel(ABC):
    """Abstract base class for all models."""
    
    @abstractmethod
    def fit(self, X: np.ndarray, y: np.ndarray) -> 'BaseModel':
        """Train the model."""
        pass
    
    @abstractmethod
    def predict(self, X: np.ndarray) -> np.ndarray:
        """Make predictions."""
        pass
    
    def predict_proba(self, X: np.ndarray) -> np.ndarray:
        """Predict class probabilities (for classification models)."""
        raise NotImplementedError("This model does not support probability predictions")


class LinearRegression(BaseModel):
    """Linear Regression implemented from scratch."""
    
    def __init__(self, learning_rate: float = 0.01, max_iterations: int = 1000, 
                 regularization: Optional[float] = None):
        """
        Initialize linear regression model.
        
        Args:
            learning_rate: Learning rate for gradient descent
            max_iterations: Maximum number of training iterations
            regularization: L2 regularization parameter (None for no regularization)
        """
        self.learning_rate = learning_rate
        self.max_iterations = max_iterations
        self.regularization = regularization
        self.weights = None
        self.bias = None
        self.costs = []
    
    def _compute_cost(self, X: np.ndarray, y: np.ndarray) -> float:
        """Compute mean squared error cost."""
        m = len(X)
        predictions = self.predict(X)
        cost = np.sum((predictions - y) ** 2) / (2 * m)
        
        if self.regularization is not None:
            reg_cost = self.regularization * np.sum(self.weights ** 2) / (2 * m)
            cost += reg_cost
            
        return cost
    
    def fit(self, X: np.ndarray, y: np.ndarray) -> 'LinearRegression':
        """Train linear regression model using gradient descent."""
        # Add bias term (intercept)
        X_with_bias = np.column_stack([np.ones(len(X)), X])
        m, n = X_with_bias.shape
        
        # Initialize parameters
        self.weights = np.random.normal(0, 0.01, n)
        self.bias = 0
        
        # Gradient descent
        for i in range(self.max_iterations):
            # Predictions
            predictions = np.dot(X_with_bias, self.weights)
            
            # Compute gradients
            error = predictions - y
            gradients = np.dot(X_with_bias.T, error) / m
            
            # Add regularization
            if self.regularization is not None:
                gradients[1:] += self.regularization * self.weights[1:] / m
            
            # Update parameters
            self.weights -= self.learning_rate * gradients
            
            # Record cost
            if i % 100 == 0:
                self.costs.append(self._compute_cost(X, y))
        
        return self
    
    def predict(self, X: np.ndarray) -> np.ndarray:
        """Make predictions."""
        if self.weights is None:
            raise ValueError("Model must be trained before making predictions")
        
        X_with_bias = np.column_stack([np.ones(len(X)), X])
        return np.dot(X_with_bias, self.weights)
    
    def r_squared(self, X: np.ndarray, y: np.ndarray) -> float:
        """Calculate R-squared score."""
        y_pred = self.predict(X)
        ss_res = np.sum((y - y_pred) ** 2)
        ss_tot = np.sum((y - np.mean(y)) ** 2)
        return 1 - (ss_res / ss_tot) if ss_tot != 0 else 0


class LogisticRegression(BaseModel):
    """Logistic Regression for binary classification."""
    
    def __init__(self, learning_rate: float = 0.01, max_iterations: int = 1000):
        """
        Initialize logistic regression.
        
        Args:
            learning_rate: Learning rate for gradient descent
            max_iterations: Maximum number of training iterations
        """
        self.learning_rate = learning_rate
        self.max_iterations = max_iterations
        self.weights = None
        self.bias = None
        self.costs = []
    
    def _sigmoid(self, z: np.ndarray) -> np.ndarray:
        """Sigmoid activation function."""
        # Clip z to prevent overflow
        z = np.clip(z, -250, 250)
        return 1 / (1 + np.exp(-z))
    
    def _compute_cost(self, X: np.ndarray, y: np.ndarray) -> float:
        """Compute logistic regression cost (cross-entropy)."""
        m = len(X)
        X_with_bias = np.column_stack([np.ones(len(X)), X])
        z = np.dot(X_with_bias, self.weights)
        predictions = self._sigmoid(z)
        
        # Cross-entropy cost
        epsilon = 1e-15  # Prevent log(0)
        predictions = np.clip(predictions, epsilon, 1 - epsilon)
        
        cost = -np.mean(y * np.log(predictions) + (1 - y) * np.log(1 - predictions))
        return cost
    
    def fit(self, X: np.ndarray, y: np.ndarray) -> 'LogisticRegression':
        """Train logistic regression model."""
        # Add bias term
        X_with_bias = np.column_stack([np.ones(len(X)), X])
        m, n = X_with_bias.shape
        
        # Initialize parameters
        self.weights = np.random.normal(0, 0.01, n)
        self.bias = 0
        
        # Gradient descent
        for i in range(self.max_iterations):
            # Predictions
            z = np.dot(X_with_bias, self.weights)
            predictions = self._sigmoid(z)
            
            # Compute gradients
            error = predictions - y
            gradients = np.dot(X_with_bias.T, error) / m
            
            # Update parameters
            self.weights -= self.learning_rate * gradients
            
            # Record cost
            if i % 100 == 0:
                self.costs.append(self._compute_cost(X, y))
        
        return self
    
    def predict(self, X: np.ndarray) -> np.ndarray:
        """Make binary predictions."""
        X_with_bias = np.column_stack([np.ones(len(X)), X])
        z = np.dot(X_with_bias, self.weights)
        probabilities = self._sigmoid(z)
        return (probabilities >= 0.5).astype(int)
    
    def predict_proba(self, X: np.ndarray) -> np.ndarray:
        """Predict class probabilities."""
        X_with_bias = np.column_stack([np.ones(len(X)), X])
        z = np.dot(X_with_bias, self.weights)
        return self._sigmoid(z)


class DecisionTreeClassifier(BaseModel):
    """Decision Tree Classifier implemented from scratch."""
    
    def __init__(self, max_depth: int = 10, min_samples_split: int = 2):
        """
        Initialize decision tree.
        
        Args:
            max_depth: Maximum depth of the tree
            min_samples_split: Minimum samples required to split a node
        """
        self.max_depth = max_depth
        self.min_samples_split = min_samples_split
        self.tree = None
        self.feature_names = None
    
    def _gini_impurity(self, y: np.ndarray) -> float:
        """Calculate Gini impurity."""
        if len(y) == 0:
            return 0
        
        _, counts = np.unique(y, return_counts=True)
        probabilities = counts / len(y)
        return 1 - np.sum(probabilities ** 2)
    
    def _information_gain(self, y: np.ndarray, left_y: np.ndarray, right_y: np.ndarray) -> float:
        """Calculate information gain from a split."""
        parent_impurity = self._gini_impurity(y)
        
        n = len(y)
        n_left, n_right = len(left_y), len(right_y)
        
        if n_left == 0 or n_right == 0:
            return 0
        
        weighted_impurity = (n_left / n) * self._gini_impurity(left_y) + \
                           (n_right / n) * self._gini_impurity(right_y)
        
        return parent_impurity - weighted_impurity
    
    def _best_split(self, X: np.ndarray, y: np.ndarray) -> Optional[Tuple[int, float]]:
        """Find the best feature and threshold for splitting."""
        best_gain = 0
        best_feature = None
        best_threshold = None
        
        n_features = X.shape[1]
        
        for feature_idx in range(n_features):
            # Get unique values for this feature
            feature_values = np.unique(X[:, feature_idx])
            
            # Try all possible thresholds (midpoints between consecutive values)
            for i in range(len(feature_values) - 1):
                threshold = (feature_values[i] + feature_values[i + 1]) / 2
                
                # Split data
                left_mask = X[:, feature_idx] <= threshold
                right_mask = ~left_mask
                
                left_y = y[left_mask]
                right_y = y[right_mask]
                
                if len(left_y) == 0 or len(right_y) == 0:
                    continue
                
                # Calculate information gain
                gain = self._information_gain(y, left_y, right_y)
                
                if gain > best_gain:
                    best_gain = gain
                    best_feature = feature_idx
                    best_threshold = threshold
        
        return (best_feature, best_threshold) if best_gain > 0 else None
    
    def _build_tree(self, X: np.ndarray, y: np.ndarray, depth: int = 0) -> dict:
        """Recursively build the decision tree."""
        # Stopping criteria
        if (depth >= self.max_depth or 
            len(y) < self.min_samples_split or 
            len(np.unique(y)) == 1):
            
            # Return leaf node
            return {
                'type': 'leaf',
                'prediction': np.bincount(y.astype(int)).argmax(),
                'samples': len(y),
                'class_counts': dict(zip(*np.unique(y, return_counts=True)))
            }
        
        # Find best split
        split_info = self._best_split(X, y)
        
        if split_info is None:
            # Return leaf if no good split found
            return {
                'type': 'leaf',
                'prediction': np.bincount(y.astype(int)).argmax(),
                'samples': len(y),
                'class_counts': dict(zip(*np.unique(y, return_counts=True)))
            }
        
        feature_idx, threshold = split_info
        
        # Split data
        left_mask = X[:, feature_idx] <= threshold
        right_mask = ~left_mask
        
        # Build subtrees
        left_subtree = self._build_tree(X[left_mask], y[left_mask], depth + 1)
        right_subtree = self._build_tree(X[right_mask], y[right_mask], depth + 1)
        
        return {
            'type': 'node',
            'feature': feature_idx,
            'threshold': threshold,
            'left': left_subtree,
            'right': right_subtree,
            'samples': len(y)
        }
    
    def fit(self, X: np.ndarray, y: np.ndarray) -> 'DecisionTreeClassifier':
        """Train decision tree."""
        self.tree = self._build_tree(X, y)
        return self
    
    def _predict_sample(self, x: np.ndarray, node: dict) -> int:
        """Predict class for a single sample."""
        if node['type'] == 'leaf':
            return node['prediction']
        
        if x[node['feature']] <= node['threshold']:
            return self._predict_sample(x, node['left'])
        else:
            return self._predict_sample(x, node['right'])
    
    def predict(self, X: np.ndarray) -> np.ndarray:
        """Make predictions."""
        if self.tree is None:
            raise ValueError("Model must be trained before making predictions")
        
        return np.array([self._predict_sample(x, self.tree) for x in X])


class KNearestNeighbors(BaseModel):
    """K-Nearest Neighbors Classifier implemented from scratch."""
    
    def __init__(self, k: int = 5):
        """
        Initialize KNN classifier.
        
        Args:
            k: Number of neighbors to consider
        """
        self.k = k
        self.X_train = None
        self.y_train = None
    
    def fit(self, X: np.ndarray, y: np.ndarray) -> 'KNearestNeighbors':
        """Store training data (no actual training needed for KNN)."""
        self.X_train = X
        self.y_train = y
        return self
    
    def _euclidean_distance(self, a: np.ndarray, b: np.ndarray) -> float:
        """Calculate Euclidean distance between two points."""
        return np.sqrt(np.sum((a - b) ** 2))
    
    def predict(self, X: np.ndarray) -> np.ndarray:
        """Make predictions using KNN."""
        if self.X_train is None:
            raise ValueError("Model must be trained before making predictions")
        
        predictions = []
        
        for x in X:
            # Calculate distances to all training points
            distances = np.array([self._euclidean_distance(x, x_train) 
                                for x_train in self.X_train])
            
            # Find k nearest neighbors
            nearest_indices = np.argsort(distances)[:self.k]
            nearest_labels = self.y_train[nearest_indices]
            
            # Vote (majority class)
            unique_labels, counts = np.unique(nearest_labels, return_counts=True)
            prediction = unique_labels[np.argmax(counts)]
            predictions.append(prediction)
        
        return np.array(predictions)


class KMeansClustering:
    """K-Means clustering algorithm implemented from scratch."""
    
    def __init__(self, k: int = 3, max_iterations: int = 100, random_state: Optional[int] = None):
        """
        Initialize K-means clustering.
        
        Args:
            k: Number of clusters
            max_iterations: Maximum number of iterations
            random_state: Random seed for reproducibility
        """
        self.k = k
        self.max_iterations = max_iterations
        self.random_state = random_state
        self.centroids = None
        self.labels = None
        self.inertia = None
    
    def _initialize_centroids(self, X: np.ndarray) -> np.ndarray:
        """Initialize centroids using k-means++."""
        if self.random_state is not None:
            np.random.seed(self.random_state)
        
        n_samples, n_features = X.shape
        
        # Choose first centroid randomly
        centroids = np.zeros((self.k, n_features))
        centroids[0] = X[np.random.randint(0, n_samples)]
        
        for c in range(1, self.k):
            # Calculate distance to nearest existing centroid
            distances = np.array([min([np.linalg.norm(x - cent) ** 2 for cent in centroids[:c]]) 
                                for x in X])
            
            # Choose next centroid with probability proportional to distance
            probabilities = distances / np.sum(distances)
            cumulative_probabilities = np.cumsum(probabilities)
            r = np.random.rand()
            
            for i, prob in enumerate(cumulative_probabilities):
                if r <= prob:
                    centroids[c] = X[i]
                    break
        
        return centroids
    
    def fit(self, X: np.ndarray) -> 'KMeansClustering':
        """Fit K-means clustering."""
        # Initialize centroids
        self.centroids = self._initialize_centroids(X)
        
        for iteration in range(self.max_iterations):
            # Assign points to nearest centroid
            distances = np.zeros((len(X), self.k))
            
            for i, centroid in enumerate(self.centroids):
                distances[:, i] = np.sum((X - centroid) ** 2, axis=1)
            
            self.labels = np.argmin(distances, axis=1)
            
            # Update centroids
            new_centroids = np.zeros_like(self.centroids)
            for i in range(self.k):
                cluster_points = X[self.labels == i]
                if len(cluster_points) > 0:
                    new_centroids[i] = np.mean(cluster_points, axis=0)
            
            # Check for convergence
            if np.allclose(self.centroids, new_centroids):
                break
            
            self.centroids = new_centroids
        
        # Calculate inertia (within-cluster sum of squares)
        self.inertia = 0
        for i in range(self.k):
            cluster_points = X[self.labels == i]
            if len(cluster_points) > 0:
                self.inertia += np.sum((cluster_points - self.centroids[i]) ** 2)
        
        return self
    
    def predict(self, X: np.ndarray) -> np.ndarray:
        """Predict cluster assignments for new data."""
        if self.centroids is None:
            raise ValueError("Model must be trained before making predictions")
        
        distances = np.zeros((len(X), self.k))
        for i, centroid in enumerate(self.centroids):
            distances[:, i] = np.sum((X - centroid) ** 2, axis=1)
        
        return np.argmin(distances, axis=1)
    
    def fit_predict(self, X: np.ndarray) -> np.ndarray:
        """Fit clustering and return cluster labels."""
        return self.fit(X).labels


class PrincipalComponentAnalysis:
    """Principal Component Analysis implemented from scratch."""
    
    def __init__(self, n_components: int = 2):
        """
        Initialize PCA.
        
        Args:
            n_components: Number of principal components to extract
        """
        self.n_components = n_components
        self.components = None
        self.explained_variance_ratio = None
        self.mean = None
    
    def fit(self, X: np.ndarray) -> 'PrincipalComponentAnalysis':
        """Fit PCA to the data."""
        # Center the data
        self.mean = np.mean(X, axis=0)
        X_centered = X - self.mean
        
        # Compute covariance matrix
        cov_matrix = np.cov(X_centered, rowvar=False)
        
        # Compute eigenvalues and eigenvectors
        eigenvalues, eigenvectors = np.linalg.eigh(cov_matrix)
        
        # Sort by eigenvalues (descending)
        indices = np.argsort(eigenvalues)[::-1]
        eigenvalues = eigenvalues[indices]
        eigenvectors = eigenvectors[:, indices]
        
        # Select top components
        self.components = eigenvectors[:, :self.n_components]
        
        # Calculate explained variance ratio
        total_variance = np.sum(eigenvalues)
        self.explained_variance_ratio = eigenvalues[:self.n_components] / total_variance
        
        return self
    
    def transform(self, X: np.ndarray) -> np.ndarray:
        """Transform data to principal component space."""
        if self.components is None:
            raise ValueError("Model must be fitted before transformation")
        
        X_centered = X - self.mean
        return np.dot(X_centered, self.components)
    
    def fit_transform(self, X: np.ndarray) -> np.ndarray:
        """Fit PCA and transform data in one step."""
        return self.fit(X).transform(X)
    
    def inverse_transform(self, X_pca: np.ndarray) -> np.ndarray:
        """Transform data back to original space."""
        if self.components is None:
            raise ValueError("Model must be fitted before inverse transformation")
        
        return np.dot(X_pca, self.components.T) + self.mean


class ModelEvaluation:
    """Model evaluation metrics and functions."""
    
    @staticmethod
    def accuracy_score(y_true: np.ndarray, y_pred: np.ndarray) -> float:
        """Calculate accuracy score."""
        return np.mean(y_true == y_pred)
    
    @staticmethod
    def precision_score(y_true: np.ndarray, y_pred: np.ndarray, average: str = 'binary') -> float:
        """Calculate precision score."""
        if average == 'binary':
            # For binary classification
            tp = np.sum((y_true == 1) & (y_pred == 1))
            fp = np.sum((y_true == 0) & (y_pred == 1))
            
            if tp + fp == 0:
                return 0.0
            return tp / (tp + fp)
        else:
            raise ValueError("Only binary precision supported in this implementation")
    
    @staticmethod
    def recall_score(y_true: np.ndarray, y_pred: np.ndarray) -> float:
        """Calculate recall score."""
        tp = np.sum((y_true == 1) & (y_pred == 1))
        fn = np.sum((y_true == 1) & (y_pred == 0))
        
        if tp + fn == 0:
            return 0.0
        return tp / (tp + fn)
    
    @staticmethod
    def f1_score(y_true: np.ndarray, y_pred: np.ndarray) -> float:
        """Calculate F1 score."""
        precision = ModelEvaluation.precision_score(y_true, y_pred)
        recall = ModelEvaluation.recall_score(y_true, y_pred)
        
        if precision + recall == 0:
            return 0.0
        return 2 * (precision * recall) / (precision + recall)
    
    @staticmethod
    def confusion_matrix(y_true: np.ndarray, y_pred: np.ndarray) -> np.ndarray:
        """Generate confusion matrix."""
        unique_classes = np.unique(np.concatenate([y_true, y_pred]))
        n_classes = len(unique_classes)
        
        cm = np.zeros((n_classes, n_classes), dtype=int)
        class_to_idx = {cls: idx for idx, cls in enumerate(unique_classes)}
        
        for true, pred in zip(y_true, y_pred):
            cm[class_to_idx[true], class_to_idx[pred]] += 1
        
        return cm
    
    @staticmethod
    def cross_validation_score(model: BaseModel, X: np.ndarray, y: np.ndarray, 
                              cv: int = 5) -> Dict:
        """Perform cross-validation and return scores."""
        scores = []
        
        # Manual cross-validation
        fold_size = len(X) // cv
        
        for i in range(cv):
            start_idx = i * fold_size
            end_idx = start_idx + fold_size if i < cv - 1 else len(X)
            
            # Create fold indices
            val_indices = np.arange(start_idx, end_idx)
            train_indices = np.concatenate([np.arange(0, start_idx), np.arange(end_idx, len(X))])
            
            # Split data
            X_train, X_val = X[train_indices], X[val_indices]
            y_train, y_val = y[train_indices], y[val_indices]
            
            # Train and evaluate
            model_copy = model.__class__()
            model_copy.fit(X_train, y_train)
            y_pred = model_copy.predict(X_val)
            
            score = ModelEvaluation.accuracy_score(y_val, y_pred)
            scores.append(score)
        
        return {
            'scores': scores,
            'mean': np.mean(scores),
            'std': np.std(scores),
            'cv': cv
        }


def demo_ml_algorithms():
    """Demonstrate machine learning algorithms."""
    print("Machine Learning Algorithms from Scratch - Educational Examples")
    print("=" * 65)
    
    # Generate sample datasets
    np.random.seed(42)
    
    # 1. Linear Regression Example
    print("\n1. Linear Regression:")
    n_samples = 100
    X_reg = np.random.randn(n_samples, 1)
    y_reg = 2 * X_reg.ravel() + 1 + 0.1 * np.random.randn(n_samples)
    
    # Split data
    X_train, X_test, y_train, y_test = DataPreprocessing.train_test_split(
        X_reg, y_reg, test_size=0.2, random_state=42)
    
    # Train model
    lr = LinearRegression(learning_rate=0.01, max_iterations=1000)
    lr.fit(X_train, y_train)
    
    # Evaluate
    y_pred = lr.predict(X_test)
    r2 = lr.r_squared(X_test, y_test)
    mse = np.mean((y_test - y_pred) ** 2)
    
    print(f"R-squared: {r2:.4f}")
    print(f"MSE: {mse:.4f}")
    
    # 2. Logistic Regression Example
    print("\n2. Logistic Regression:")
    n_samples = 200
    X_clf = np.random.randn(n_samples, 2)
    y_clf = ((X_clf[:, 0] + X_clf[:, 1]) > 0).astype(int)
    
    # Split data
    X_train, X_test, y_train, y_test = DataPreprocessing.train_test_split(
        X_clf, y_clf, test_size=0.2, random_state=42)
    
    # Standardize features
    X_train_std, _, _ = DataPreprocessing.standardize(X_train)
    X_test_std, _, _ = DataPreprocessing.standardize(X_test)
    
    # Train model
    log_reg = LogisticRegression(learning_rate=0.1, max_iterations=1000)
    log_reg.fit(X_train_std, y_train)
    
    # Evaluate
    y_pred = log_reg.predict(X_test_std)
    accuracy = ModelEvaluation.accuracy_score(y_test, y_pred)
    precision = ModelEvaluation.precision_score(y_test, y_pred)
    recall = ModelEvaluation.recall_score(y_test, y_pred)
    f1 = ModelEvaluation.f1_score(y_test, y_pred)
    
    print(f"Accuracy:  {accuracy:.4f}")
    print(f"Precision: {precision:.4f}")
    print(f"Recall:    {recall:.4f}")
    print(f"F1-Score:  {f1:.4f}")
    
    # 3. Decision Tree Example
    print("\n3. Decision Tree:")
    dt = DecisionTreeClassifier(max_depth=5, min_samples_split=5)
    dt.fit(X_train_std, y_train)
    
    y_pred = dt.predict(X_test_std)
    accuracy = ModelEvaluation.accuracy_score(y_test, y_pred)
    print(f"Decision Tree Accuracy: {accuracy:.4f}")
    
    # 4. K-Nearest Neighbors Example
    print("\n4. K-Nearest Neighbors:")
    knn = KNearestNeighbors(k=5)
    knn.fit(X_train_std, y_train)
    
    y_pred = knn.predict(X_test_std)
    accuracy = ModelEvaluation.accuracy_score(y_test, y_pred)
    print(f"KNN Accuracy: {accuracy:.4f}")
    
    # 5. K-Means Clustering Example
    print("\n5. K-Means Clustering:")
    X_cluster = np.random.randn(150, 2)
    X_cluster[:50] += np.array([2, 2])  # First cluster
    X_cluster[50:100] += np.array([-2, 2])  # Second cluster
    X_cluster[100:] += np.array([0, -2])  # Third cluster
    
    kmeans = KMeansClustering(k=3, random_state=42)
    labels = kmeans.fit_predict(X_cluster)
    
    print(f"Clustering inertia: {kmeans.inertia:.4f}")
    
    # 6. PCA Example
    print("\n6. Principal Component Analysis:")
    X_pca = np.random.randn(100, 5)
    X_pca[:50, 0] += 3  # Add some structure
    X_pca[50:, 1] += 2
    
    pca = PrincipalComponentAnalysis(n_components=2)
    X_pca_transformed = pca.fit_transform(X_pca)
    
    print(f"Explained variance ratio: {pca.explained_variance_ratio}")
    print(f"Total explained variance: {np.sum(pca.explained_variance_ratio):.4f}")
    
    # 7. Cross-Validation Example
    print("\n7. Cross-Validation:")
    cv_results = ModelEvaluation.cross_validation_score(
        LogisticRegression(learning_rate=0.1, max_iterations=500), 
        X_train_std, y_train, cv=5)
    
    print(f"Cross-validation scores: {cv_results['scores']}")
    print(f"Mean accuracy: {cv_results['mean']:.4f} ± {cv_results['std']:.4f}")
    
    print("\nNote: Visualization plots require matplotlib backend to display.")
    print("Use the transformed data and predictions to create plots when running interactively.")


if __name__ == "__main__":
    demo_ml_algorithms()
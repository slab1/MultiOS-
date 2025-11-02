"""
Computer Vision Module for Educational Robotics

Provides camera interface, line detection, and object detection capabilities
"""

import numpy as np
import cv2
import time
from typing import List, Tuple, Optional, Dict, Any
from dataclasses import dataclass
import threading


@dataclass
class VisionResult:
    """Container for vision processing results"""
    detected_objects: List[Dict[str, Any]]
    lines: List[Tuple[float, float, float]]  # (rho, theta, confidence)
    color_mask: Optional[np.ndarray] = None
    processed_image: Optional[np.ndarray] = None
    timestamp: float = 0.0
    
    def __post_init__(self):
        if self.timestamp == 0.0:
            self.timestamp = time.time()


class Camera:
    """Camera interface for robot vision"""
    
    def __init__(self, camera_index: int = 0, width: int = 640, height: int = 480):
        self.camera_index = camera_index
        self.width = width
        self.height = height
        self.camera = None
        self.is_running = False
        self.latest_frame = None
        self._capture_thread = None
        self.frame_count = 0
        
        # Camera parameters
        self.fps = 30
        self.exposure = 0
        self.brightness = 50
        self.contrast = 50
        
    def initialize(self) -> bool:
        """Initialize camera"""
        try:
            # Try to open camera with OpenCV
            self.camera = cv2.VideoCapture(self.camera_index)
            
            if not self.camera.isOpened():
                print(f"Could not open camera {self.camera_index}")
                # Simulation mode - create dummy frames
                self._start_simulation_mode()
                return True
                
            # Set camera properties
            self.camera.set(cv2.CAP_PROP_FRAME_WIDTH, self.width)
            self.camera.set(cv2.CAP_PROP_FRAME_HEIGHT, self.height)
            self.camera.set(cv2.CAP_PROP_FPS, self.fps)
            
            # Test capture
            ret, frame = self.camera.read()
            if ret:
                print(f"Camera {self.camera_index} initialized successfully")
                print(f"Resolution: {self.width}x{self.height}, FPS: {self.fps}")
                return True
            else:
                print("Camera initialization failed - starting simulation mode")
                self._start_simulation_mode()
                return True
                
        except Exception as e:
            print(f"Camera initialization error: {e}")
            print("Starting simulation mode")
            self._start_simulation_mode()
            return True
            
    def _start_simulation_mode(self):
        """Start simulation mode for testing without camera"""
        self.camera = None
        print("Camera simulation mode enabled")
        
    def start_capture(self) -> bool:
        """Start continuous capture"""
        if self.is_running:
            return True
            
        self.is_running = True
        
        if self.camera:
            self._capture_thread = threading.Thread(target=self._capture_loop, daemon=True)
            self._capture_thread.start()
        else:
            # Simulation mode
            self._capture_thread = threading.Thread(target=self._simulation_loop, daemon=True)
            self._capture_thread.start()
            
        print("Camera capture started")
        return True
        
    def stop_capture(self):
        """Stop continuous capture"""
        self.is_running = False
        
        if self._capture_thread and self._capture_thread.is_alive():
            self._capture_thread.join(timeout=1.0)
            
        if self.camera:
            self.camera.release()
            
        print("Camera capture stopped")
        
    def _capture_loop(self):
        """Continuous capture loop"""
        while self.is_running:
            try:
                ret, frame = self.camera.read()
                if ret:
                    self.latest_frame = frame
                    self.frame_count += 1
                else:
                    print("Failed to capture frame")
                    time.sleep(0.1)
            except Exception as e:
                print(f"Capture error: {e}")
                time.sleep(0.1)
                
    def _simulation_loop(self):
        """Simulation loop for testing without physical camera"""
        while self.is_running:
            # Generate simulated frame
            frame = self._generate_simulated_frame()
            self.latest_frame = frame
            self.frame_count += 1
            time.sleep(1.0 / self.fps)
            
    def _generate_simulated_frame(self) -> np.ndarray:
        """Generate simulated camera frame"""
        # Create base frame
        frame = np.ones((self.height, self.width, 3), dtype=np.uint8) * 128
        
        # Add some structure for testing
        # White line in center (for line following demo)
        center_y = self.height // 2
        line_width = 20
        frame[center_y - line_width//2:center_y + line_width//2, :] = [255, 255, 255]
        
        # Add some colored objects for testing
        # Red rectangle
        cv2.rectangle(frame, (100, 100), (200, 200), (0, 0, 255), -1)
        
        # Green circle
        cv2.circle(frame, (400, 300), 50, (0, 255, 0), -1)
        
        # Blue text
        cv2.putText(frame, "SIMULATION", (50, 450), cv2.FONT_HERSHEY_SIMPLEX, 
                   1, (255, 0, 0), 2, cv2.LINE_AA)
        
        # Add some noise
        noise = np.random.normal(0, 10, frame.shape).astype(np.uint8)
        frame = cv2.add(frame, noise)
        
        return frame
        
    def capture_frame(self) -> Optional[np.ndarray]:
        """Capture single frame"""
        if self.latest_frame is not None:
            return self.latest_frame.copy()
            
        if self.camera:
            ret, frame = self.camera.read()
            if ret:
                return frame
                
        # Return simulated frame
        return self._generate_simulated_frame()
        
    def get_frame_info(self) -> Dict[str, Any]:
        """Get camera frame information"""
        return {
            'width': self.width,
            'height': self.height,
            'fps': self.fps,
            'frame_count': self.frame_count,
            'is_capturing': self.is_running,
            'timestamp': time.time()
        }
        
    def set_parameters(self, **kwargs):
        """Set camera parameters"""
        if self.camera:
            for param, value in kwargs.items():
                if param == 'brightness':
                    self.camera.set(cv2.CAP_PROP_BRIGHTNESS, value)
                elif param == 'contrast':
                    self.camera.set(cv2.CAP_PROP_CONTRAST, value)
                elif param == 'exposure':
                    self.camera.set(cv2.CAP_PROP_EXPOSURE, value)
                elif param == 'fps':
                    self.fps = value
                    self.camera.set(cv2.CAP_PROP_FPS, value)
                    
        for param, value in kwargs.items():
            if hasattr(self, param):
                setattr(self, param, value)


class ColorDetector:
    """Color-based object detection"""
    
    def __init__(self):
        # Color ranges in HSV (H, S, V) - [min, max]
        self.color_ranges = {
            'red': [
                ([0, 50, 50], [10, 255, 255]),
                ([170, 50, 50], [180, 255, 255])
            ],
            'green': [([60, 50, 50], [80, 255, 255])],
            'blue': [([110, 50, 50], [130, 255, 255])],
            'yellow': [([25, 50, 50], [35, 255, 255])],
            'white': [([0, 0, 200], [180, 30, 255])],
            'black': [([0, 0, 0], [180, 255, 50])]
        }
        
    def detect_colors(self, image: np.ndarray, colors: Optional[List[str]] = None) -> List[Dict[str, Any]]:
        """Detect colored objects in image"""
        if colors is None:
            colors = list(self.color_ranges.keys())
            
        hsv = cv2.cvtColor(image, cv2.COLOR_BGR2HSV)
        detected_objects = []
        
        for color_name in colors:
            if color_name not in self.color_ranges:
                continue
                
            # Create mask for color
            mask = np.zeros(hsv.shape[:2], dtype=np.uint8)
            
            for color_range in self.color_ranges[color_name]:
                lower = np.array(color_range[0], dtype=np.uint8)
                upper = np.array(color_range[1], dtype=np.uint8)
                color_mask = cv2.inRange(hsv, lower, upper)
                mask = cv2.bitwise_or(mask, color_mask)
                
            # Find contours
            contours, _ = cv2.findContours(mask, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)
            
            for contour in contours:
                area = cv2.contourArea(contour)
                if area > 100:  # Minimum area threshold
                    # Get bounding rectangle
                    x, y, w, h = cv2.boundingRect(contour)
                    center_x = x + w // 2
                    center_y = y + h // 2
                    
                    detected_objects.append({
                        'color': color_name,
                        'area': area,
                        'center': (center_x, center_y),
                        'bbox': (x, y, w, h),
                        'confidence': min(1.0, area / 1000.0)
                    })
                    
        return detected_objects


class LineDetector:
    """Line detection using Hough Transform and computer vision techniques"""
    
    def __init__(self):
        self.edge_threshold = 50
        self.min_line_length = 50
        self.max_line_gap = 10
        
    def detect_lines(self, image: np.ndarray) -> Tuple[List[Tuple[float, float, float]], np.ndarray]:
        """
        Detect lines in image using Hough Line Transform
        
        Returns:
            Tuple of (lines, processed_image)
            Lines are (rho, theta, confidence) tuples
        """
        # Convert to grayscale if needed
        if len(image.shape) == 3:
            gray = cv2.cvtColor(image, cv2.COLOR_BGR2GRAY)
        else:
            gray = image.copy()
            
        # Apply Gaussian blur
        blurred = cv2.GaussianBlur(gray, (5, 5), 0)
        
        # Edge detection
        edges = cv2.Canny(blurred, 50, 150, apertureSize=3)
        
        # Morphological operations to clean up edges
        kernel = np.ones((3, 3), np.uint8)
        edges = cv2.morphologyEx(edges, cv2.MORPH_CLOSE, kernel)
        
        # Hough Line Transform
        lines = cv2.HoughLinesP(
            edges,
            1,  # rho resolution
            np.pi / 180,  # theta resolution
            threshold=30,  # minimum number of intersections
            lines=np.array([]),
            minLineLength=self.min_line_length,
            maxLineGap=self.max_line_gap
        )
        
        detected_lines = []
        processed_image = np.copy(image)
        
        if lines is not None:
            for line in lines:
                x1, y1, x2, y2 = line[0]
                
                # Calculate line parameters
                rho = abs(x2 * y1 - x1 * y2) / np.sqrt((y2 - y1)**2 + (x2 - x1)**2)
                theta = np.arctan2(y2 - y1, x2 - x1)
                
                # Calculate confidence based on line length
                line_length = np.sqrt((x2 - x1)**2 + (y2 - y1)**2)
                confidence = min(1.0, line_length / 200.0)
                
                detected_lines.append((rho, theta, confidence))
                
                # Draw line on processed image
                cv2.line(processed_image, (x1, y1), (x2, y2), (0, 255, 0), 2)
                
        return detected_lines, processed_image
        
    def detect_line_following_line(self, image: np.ndarray) -> Optional[Dict[str, Any]]:
        """
        Detect line for line-following robot
        
        Returns:
            Dictionary with line information or None if no line detected
        """
        # Convert to HSV for better color segmentation
        hsv = cv2.cvtColor(image, cv2.COLOR_BGR2HSV)
        
        # Define range for black/white line detection
        # For black line on white surface
        lower_line = np.array([0, 0, 0])
        upper_line = np.array([180, 255, 80])
        
        # For white line on dark surface  
        lower_line_white = np.array([0, 0, 200])
        upper_line_white = np.array([180, 30, 255])
        
        # Create masks
        mask_black = cv2.inRange(hsv, lower_line, upper_line)
        mask_white = cv2.inRange(hsv, lower_line_white, upper_line_white)
        mask = cv2.bitwise_or(mask_black, mask_white)
        
        # Apply morphological operations
        kernel = np.ones((5, 5), np.uint8)
        mask = cv2.morphologyEx(mask, cv2.MORPH_CLOSE, kernel)
        mask = cv2.morphologyEx(mask, cv2.MORPH_OPEN, kernel)
        
        # Find contours
        contours, _ = cv2.findContours(mask, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)
        
        if not contours:
            return None
            
        # Find the largest contour (main line)
        largest_contour = max(contours, key=cv2.contourArea)
        area = cv2.contourArea(largest_contour)
        
        if area < 100:  # Minimum area threshold
            return None
            
        # Get line properties
        x, y, w, h = cv2.boundingRect(largest_contour)
        center_x = x + w // 2
        center_y = y + h // 2
        
        # Calculate line angle using moments
        moments = cv2.moments(largest_contour)
        if moments['m00'] != 0:
            # Calculate angle using second moments
            mu20 = moments['mu20']
            mu02 = moments['mu02']
            mu11 = moments['mu11']
            
            angle = 0.5 * np.arctan2(2 * mu11, mu20 - mu02)
        else:
            # Use simple bounding rectangle angle
            angle = np.arctan2(h, w)
            
        # Determine line type (horizontal, vertical, diagonal)
        line_type = 'unknown'
        if abs(angle) < np.pi/8 or abs(angle - np.pi) < np.pi/8:
            line_type = 'horizontal'
        elif abs(angle - np.pi/2) < np.pi/8 or abs(angle + np.pi/2) < np.pi/8:
            line_type = 'vertical'
        else:
            line_type = 'diagonal'
            
        # Calculate line confidence
        image_area = image.shape[0] * image.shape[1]
        confidence = min(1.0, area / (image_area * 0.1))  # Normalize by image area
        
        return {
            'center': (center_x, center_y),
            'angle': angle,
            'line_type': line_type,
            'confidence': confidence,
            'area': area,
            'bbox': (x, y, w, h),
            'mask': mask
        }
        
    def visualize_line_detection(self, image: np.ndarray, line_info: Optional[Dict[str, Any]]) -> np.ndarray:
        """Visualize line detection results"""
        if line_info is None:
            return image
            
        result_image = image.copy()
        
        # Draw center point
        center = line_info['center']
        cv2.circle(result_image, center, 10, (0, 255, 255), -1)
        cv2.putText(result_image, 'Line Center', 
                   (center[0] + 15, center[1]), 
                   cv2.FONT_HERSHEY_SIMPLEX, 0.7, (0, 255, 255), 2)
        
        # Draw line direction
        angle = line_info['angle']
        length = 50
        end_point = (
            int(center[0] + length * np.cos(angle)),
            int(center[1] + length * np.sin(angle))
        )
        cv2.arrowedLine(result_image, center, end_point, (255, 0, 255), 3)
        
        # Draw text with line information
        info_text = f"Type: {line_info['line_type']}"
        info_text += f" Angle: {np.degrees(angle):.1f}°"
        info_text += f" Conf: {line_info['confidence']:.2f}"
        
        cv2.putText(result_image, info_text, (10, 30), 
                   cv2.FONT_HERSHEY_SIMPLEX, 0.6, (0, 255, 0), 2)
        
        return result_image


class ObjectDetector:
    """Object detection using computer vision techniques"""
    
    def __init__(self):
        self.color_detector = ColorDetector()
        
    def detect_objects(self, image: np.ndarray) -> List[Dict[str, Any]]:
        """Detect objects in image using multiple methods"""
        detected_objects = []
        
        # Color-based detection
        color_objects = self.color_detector.detect_colors(image)
        for obj in color_objects:
            obj['detection_method'] = 'color'
            detected_objects.append(obj)
            
        # Shape-based detection
        shape_objects = self._detect_shapes(image)
        for obj in shape_objects:
            obj['detection_method'] = 'shape'
            detected_objects.append(obj)
            
        # Template matching for common objects
        template_objects = self._template_match(image)
        for obj in template_objects:
            obj['detection_method'] = 'template'
            detected_objects.append(obj)
            
        return detected_objects
        
    def _detect_shapes(self, image: np.ndarray) -> List[Dict[str, Any]]:
        """Detect basic shapes (circles, rectangles, triangles)"""
        # Convert to grayscale
        gray = cv2.cvtColor(image, cv2.COLOR_BGR2GRAY)
        
        # Apply threshold
        _, thresh = cv2.threshold(gray, 127, 255, cv2.THRESH_BINARY)
        
        # Find contours
        contours, _ = cv2.findContours(thresh, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)
        
        detected_shapes = []
        
        for contour in contours:
            area = cv2.contourArea(contour)
            if area < 100:  # Minimum area
                continue
                
            # Approximate contour
            epsilon = 0.02 * cv2.arcLength(contour, True)
            approx = cv2.approxPolyDP(contour, epsilon, True)
            
            x, y, w, h = cv2.boundingRect(contour)
            center_x = x + w // 2
            center_y = y + h // 2
            
            # Classify shape based on number of vertices
            shape_type = 'unknown'
            if len(approx) == 3:
                shape_type = 'triangle'
            elif len(approx) == 4:
                # Check if it's a square or rectangle
                aspect_ratio = float(w) / h
                if 0.8 <= aspect_ratio <= 1.2:
                    shape_type = 'square'
                else:
                    shape_type = 'rectangle'
            elif len(approx) > 10:
                shape_type = 'circle'
                
            detected_shapes.append({
                'type': shape_type,
                'center': (center_x, center_y),
                'area': area,
                'bbox': (x, y, w, h),
                'confidence': min(1.0, area / 1000.0)
            })
            
        return detected_shapes
        
    def _template_match(self, image: np.ndarray) -> List[Dict[str, Any]]:
        """Template matching for common objects"""
        # This is a simplified version - in practice, you'd load templates
        detected_objects = []
        
        # Example: look for red circles (simple template matching)
        hsv = cv2.cvtColor(image, cv2.COLOR_BGR2HSV)
        
        # Red color range
        lower_red1 = np.array([0, 50, 50])
        upper_red1 = np.array([10, 255, 255])
        lower_red2 = np.array([170, 50, 50])
        upper_red2 = np.array([180, 255, 255])
        
        mask1 = cv2.inRange(hsv, lower_red1, upper_red1)
        mask2 = cv2.inRange(hsv, lower_red2, upper_red2)
        mask = cv2.bitwise_or(mask1, mask2)
        
        # Find circles using HoughCircles
        circles = cv2.HoughCircles(
            mask, cv2.HOUGH_GRADIENT, 1, 20,
            param1=50, param2=30, minRadius=10, maxRadius=100
        )
        
        if circles is not None:
            circles = np.round(circles[0, :]).astype("int")
            for (x, y, r) in circles:
                detected_objects.append({
                    'type': 'red_circle',
                    'center': (x, y),
                    'radius': r,
                    'confidence': 0.8
                })
                
        return detected_objects
        
    def visualize_detection(self, image: np.ndarray, objects: List[Dict[str, Any]]) -> np.ndarray:
        """Visualize detected objects"""
        result_image = image.copy()
        
        for obj in objects:
            # Draw bounding box or circle
            if 'bbox' in obj:
                x, y, w, h = obj['bbox']
                cv2.rectangle(result_image, (x, y), (x + w, y + h), (0, 255, 0), 2)
            elif 'center' in obj and 'radius' in obj:
                center = obj['center']
                radius = obj['radius']
                cv2.circle(result_image, center, radius, (0, 255, 0), 2)
                
            # Draw label
            if 'center' in obj:
                center = obj['center']
                label = f"{obj.get('color', obj.get('type', 'Object'))}"
                confidence = obj.get('confidence', 0.0)
                label += f" {confidence:.2f}"
                
                cv2.putText(result_image, label, 
                           (center[0] - 20, center[1] - 20), 
                           cv2.FONT_HERSHEY_SIMPLEX, 0.5, (0, 255, 0), 1)
                           
        return result_image


class VisionProcessor:
    """Main vision processing class that combines all vision capabilities"""
    
    def __init__(self, camera: Optional[Camera] = None):
        self.camera = camera or Camera()
        self.line_detector = LineDetector()
        self.object_detector = ObjectDetector()
        self.color_detector = ColorDetector()
        self.processing_enabled = True
        
        # Processing results cache
        self.last_result = None
        self.processing_stats = {
            'total_frames': 0,
            'processing_time_total': 0.0,
            'line_detections': 0,
            'object_detections': 0
        }
        
    def start_processing(self) -> bool:
        """Start vision processing"""
        if not self.camera.is_running:
            self.camera.start_capture()
            
        return True
        
    def stop_processing(self):
        """Stop vision processing"""
        self.processing_enabled = False
        self.camera.stop_capture()
        
    def process_frame(self, image: Optional[np.ndarray] = None) -> VisionResult:
        """Process single frame and return results"""
        start_time = time.time()
        
        # Get frame
        if image is None:
            image = self.camera.capture_frame()
            
        if image is None:
            return VisionResult([], [])
            
        result = VisionResult([], [])
        result.processed_image = image.copy()
        
        # Process line detection
        if self.processing_enabled:
            line_result, processed_img = self.line_detector.detect_lines(image)
            result.lines = line_result
            if processed_img is not None:
                result.processed_image = processed_img
                
            if line_result:
                self.processing_stats['line_detections'] += 1
                
            # Process object detection
            objects = self.object_detector.detect_objects(image)
            result.detected_objects = objects
            if objects:
                self.processing_stats['object_detections'] += 1
                
        # Update statistics
        processing_time = time.time() - start_time
        self.processing_stats['total_frames'] += 1
        self.processing_stats['processing_time_total'] += processing_time
        
        self.last_result = result
        return result
        
    def detect_line_for_following(self, image: Optional[np.ndarray] = None) -> Optional[Dict[str, Any]]:
        """Specialized method for line-following robots"""
        if image is None:
            image = self.camera.capture_frame()
            
        return self.line_detector.detect_line_following_line(image)
        
    def get_processing_stats(self) -> Dict[str, Any]:
        """Get processing statistics"""
        stats = self.processing_stats.copy()
        
        if stats['total_frames'] > 0:
            stats['avg_processing_time'] = stats['processing_time_total'] / stats['total_frames']
            stats['fps'] = 1.0 / stats['avg_processing_time'] if stats['avg_processing_time'] > 0 else 0
        else:
            stats['avg_processing_time'] = 0
            stats['fps'] = 0
            
        return stats
        
    def save_frame(self, filename: str, image: Optional[np.ndarray] = None):
        """Save current frame to file"""
        if image is None:
            image = self.camera.capture_frame()
            
        if image is not None:
            cv2.imwrite(filename, image)
            print(f"Frame saved to {filename}")


# Example usage and testing
if __name__ == "__main__":
    print("Testing Computer Vision Module...")
    
    # Test camera
    print("\n1. Testing Camera...")
    camera = Camera(width=640, height=480)
    camera.initialize()
    camera.start_capture()
    
    # Test vision processing
    print("\n2. Testing Vision Processing...")
    processor = VisionProcessor(camera)
    processor.start_processing()
    
    # Capture and process a few frames
    for i in range(3):
        frame = camera.capture_frame()
        if frame is not None:
            result = processor.process_frame(frame)
            print(f"Frame {i}: {len(result.detected_objects)} objects, {len(result.lines)} lines")
            
    # Test line following detection
    print("\n3. Testing Line Following Detection...")
    frame = camera.capture_frame()
    if frame is not None:
        line_info = processor.detect_line_for_following(frame)
        if line_info:
            print(f"Line detected: {line_info['line_type']} at {line_info['center']}")
        else:
            print("No line detected")
            
    # Print statistics
    print("\n4. Processing Statistics:")
    stats = processor.get_processing_stats()
    for key, value in stats.items():
        print(f"  {key}: {value}")
        
    # Cleanup
    camera.stop_capture()
    processor.stop_processing()
    
    print("\nComputer vision testing complete!")

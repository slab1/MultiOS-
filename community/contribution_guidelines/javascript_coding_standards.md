# JavaScript Coding Standards

This document defines the coding standards and style guidelines for all JavaScript and TypeScript code in the MultiOS project. These standards ensure code quality, maintainability, and consistency for web tools, documentation sites, and interactive demos.

## 📋 Table of Contents

- [Code Formatting](#code-formatting)
- [Naming Conventions](#naming-conventions)
- [TypeScript Integration](#typescript-integration)
- [Error Handling](#error-handling)
- [Performance Guidelines](#performance-guidelines)
- [Testing Standards](#testing-standards)
- [Package Management](#package-management)
- [Framework Guidelines](#framework-guidelines)
- [Documentation](#documentation)
- [Linting and CI](#linting-and-ci)

## 🎨 Code Formatting

### ESLint and Prettier Configuration

All JavaScript/TypeScript code must follow the project's formatting standards:

**ESLint Configuration (`.eslintrc.js`):**
```javascript
module.exports = {
  env: {
    browser: true,
    es2021: true,
    node: true,
  },
  extends: [
    'eslint:recommended',
    '@typescript-eslint/recommended',
    'prettier',
  ],
  parser: '@typescript-eslint/parser',
  parserOptions: {
    ecmaVersion: 'latest',
    sourceType: 'module',
    ecmaFeatures: {
      jsx: true,
    },
  },
  plugins: [
    '@typescript-eslint',
    'react-hooks',
    'import',
  ],
  rules: {
    // TypeScript specific rules
    '@typescript-eslint/no-unused-vars': 'error',
    '@typescript-eslint/explicit-function-return-type': 'warn',
    '@typescript-eslint/no-explicit-any': 'warn',
    '@typescript-eslint/no-non-null-assertion': 'error',
    
    // General code quality
    'prefer-const': 'error',
    'no-var': 'error',
    'no-console': 'warn',
    'no-debugger': 'error',
    
    // Import rules
    'import/no-duplicates': 'error',
    'import/order': [
      'error',
      {
        groups: [
          'builtin',
          'external',
          'internal',
          'parent',
          'sibling',
          'index',
        ],
        'newlines-between': 'always',
      },
    ],
  },
  settings: {
    'import/resolver': {
      typescript: {},
    },
  },
};
```

**Prettier Configuration (`.prettierrc.js`):**
```javascript
module.exports = {
  semi: true,
  trailingComma: 'es5',
  singleQuote: true,
  printWidth: 80,
  tabWidth: 2,
  useTabs: false,
  quoteProps: 'as-needed',
  jsxSingleQuote: true,
  bracketSpacing: true,
  bracketSameLine: false,
  arrowParens: 'avoid',
  endOfLine: 'lf',
};
```

### Code Structure

```typescript
// Core imports first
import { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';

// Third-party library imports
import { motion } from 'framer-motion';
import { toast } from 'react-hot-toast';
import { debounce } from 'lodash';

// Internal module imports
import { MemoryRegion } from '../types/memory';
import { MemoryAnalyzer } from '../services/analyzer';
import { formatBytes, formatAddress } from '../utils/formatters';

// Constants and types
interface MemoryAnalysisResult {
  totalSize: number;
  usedSize: number;
  regions: MemoryRegion[];
  fragmentationRatio: number;
}

const DEFAULT_PAGE_SIZE = 4096;
const MAX_MEMORY_SIZE = 0xFFFF_FFFF;

// Component definition
export const MemoryVisualizer: React.FC<MemoryVisualizerProps> = ({
  memoryMap,
  onRegionSelect,
}) => {
  // State management
  const [selectedRegion, setSelectedRegion] = useState<MemoryRegion | null>(null);
  const [analysisResult, setAnalysisResult] = useState<MemoryAnalysisResult | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  
  // Effects
  useEffect(() => {
    if (memoryMap.length > 0) {
      analyzeMemory();
    }
  }, [memoryMap]);
  
  // Event handlers
  const handleRegionClick = (region: MemoryRegion) => {
    setSelectedRegion(region);
    onRegionSelect?.(region);
  };
  
  // Async operations
  const analyzeMemory = async () => {
    setIsLoading(true);
    try {
      const analyzer = new MemoryAnalyzer();
      const result = await analyzer.analyze(memoryMap);
      setAnalysisResult(result);
    } catch (error) {
      console.error('Memory analysis failed:', error);
      toast.error('Failed to analyze memory regions');
    } finally {
      setIsLoading(false);
    }
  };
  
  // Render logic
  if (isLoading) {
    return <LoadingSpinner message="Analyzing memory regions..." />;
  }
  
  return (
    <div className="memory-visualizer">
      <div className="visualization-header">
        <h2>Memory Regions Visualization</h2>
        {analysisResult && (
          <MemoryStats stats={analysisResult} />
        )}
      </div>
      
      <div className="memory-regions">
        {memoryMap.map((region) => (
          <MemoryRegionComponent
            key={region.address}
            region={region}
            isSelected={selectedRegion?.address === region.address}
            onClick={() => handleRegionClick(region)}
          />
        ))}
      </div>
      
      {selectedRegion && (
        <MemoryRegionDetails region={selectedRegion} />
      )}
    </div>
  );
};
```

## 🏷️ Naming Conventions

### Variables and Functions

```typescript
// Variables and functions: camelCase
const maxBufferSize = 4096;
let currentPointer = 0x1000;

function calculateChecksum(data: Uint8Array): number {
  let checksum = 0;
  for (let i = 0; i < data.length; i++) {
    checksum = (checksum + data[i]) & 0xFF;
  }
  return checksum;
}

const initializeMemoryManager = async (config: MemoryConfig): Promise<boolean> => {
  // Implementation
  return true;
};

// Event handlers
const handleMemoryRegionClick = (event: MouseEvent, region: MemoryRegion): void => {
  // Handle click event
};

const handleDataReceived = useCallback((data: NetworkData) => {
  processIncomingData(data);
}, [processIncomingData]);
```

### Classes and Types

```typescript
// Classes: PascalCase
export class MemoryRegion {
  public readonly address: number;
  public readonly size: number;
  public readonly permissions: PermissionFlags;
  public readonly name?: string;
  
  constructor(address: number, size: number, permissions: PermissionFlags, name?: string) {
    this.address = address;
    this.size = size;
    this.permissions = permissions;
    this.name = name;
  }
  
  public get isExecutable(): boolean {
    return (this.permissions & PermissionFlags.Execute) !== 0;
  }
  
  public get endAddress(): number {
    return this.address + this.size;
  }
}

export class NetworkPacket {
  private readonly header: PacketHeader;
  private readonly payload: Uint8Array;
  
  constructor(header: PacketHeader, payload: Uint8Array) {
    this.header = header;
    this.payload = payload;
  }
  
  public get totalLength(): number {
    return this.header.size + this.payload.length;
  }
}
```

### Constants and Enums

```typescript
// Constants: UPPER_CASE or camelCase with appropriate constants
const MAX_MEMORY_SIZE = 0xFFFF_FFFF;
const DEFAULT_PAGE_SIZE = 4096;
const NETWORK_BUFFER_SIZE = 1500;

// Object with frozen properties
const MEMORY_TYPES = Object.freeze({
  KERNEL: 'kernel',
  USER: 'user',
  DEVICE: 'device',
  DMA: 'dma',
} as const);

// Enum-style objects
const PermissionFlags = {
  Read: 0x1,
  Write: 0x2,
  Execute: 0x4,
} as const;

type PermissionFlag = typeof PermissionFlags[keyof typeof PermissionFlags];

// Configuration objects
const defaultConfig = Object.freeze({
  bufferSize: 1024,
  timeout: 5000,
  retries: 3,
});
```

### File and Module Naming

```typescript
// File names: kebab-case (same as module names)
// memory-analyzer.ts
// network-utils.ts
// device-drivers.ts

// Component files: PascalCase with component suffix
// MemoryVisualizer.tsx
// NetworkInterface.tsx
// DeviceStatus.tsx

// Service files: camelCase with service suffix
// memoryService.ts
// networkService.ts
// analyticsService.ts
```

### Private vs Public

```typescript
export class MemoryManager {
  // Public properties
  public readonly isInitialized: boolean;
  public readonly totalMemory: number;
  
  // Private properties (convention)
  private readonly regions: Map<string, MemoryRegion>;
  private _currentUsage: number;
  
  // Protected properties (convention)
  protected readonly config: MemoryManagerConfig;
  
  // Public methods
  public async initialize(): Promise<boolean> {
    // Implementation
    return true;
  }
  
  public allocate(size: number, alignment: number = 1): Promise<MemoryAllocation | null> {
    // Implementation
    return null;
  }
  
  // Private methods
  private findSuitableRegion(size: number, alignment: number): MemoryRegion | null {
    // Implementation
    return null;
  }
  
  // Protected methods (for inheritance)
  protected validateAllocation(size: number): boolean {
    return size > 0 && size <= this.totalMemory;
  }
  
  // Getters
  public get currentUsage(): number {
    return this._currentUsage;
  }
}
```

## 🔍 TypeScript Integration

### Type Definitions

```typescript
// Basic types
type MemoryAddress = number;
type PortNumber = number;
type NetworkProtocol = 'tcp' | 'udp' | 'icmp';

interface MemoryRegion {
  address: MemoryAddress;
  size: number;
  permissions: PermissionFlags;
  name?: string;
}

interface NetworkPacket {
  source: NetworkEndpoint;
  destination: NetworkEndpoint;
  protocol: NetworkProtocol;
  data: Uint8Array;
  timestamp: number;
}

// Generic types
interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
  timestamp: number;
}

interface PaginatedResponse<T> extends ApiResponse<T[]> {
  pagination: {
    page: number;
    pageSize: number;
    total: number;
    hasMore: boolean;
  };
}

// Utility types
type ReadonlyMemoryRegion = Readonly<MemoryRegion>;
type OptionalMemoryRegion = MemoryRegion | null;
type MemoryRegionList = ReadonlyArray<MemoryRegion>;

// Advanced type patterns
interface EventHandler<T = any> {
  (event: T): void | Promise<void>;
}

interface EventEmitter<T = any> {
  on(event: string, handler: EventHandler<T>): void;
  off(event: string, handler: EventHandler<T>): void;
  emit(event: string, data: T): void;
}

// Conditional types
type ResponseData<T> = T extends { success: true } ? T['data'] : never;

type Result<T, E = Error> = 
  | { success: true; data: T }
  | { success: false; error: E };
```

### Interface vs Type Aliases

```typescript
// Use interfaces for object types that will be extended
interface DeviceDriver {
  name: string;
  version: string;
  initialize(): Promise<boolean>;
  read(address: number, size: number): Promise<Uint8Array>;
  write(address: number, data: Uint8Array): Promise<void>;
  cleanup(): Promise<void>;
}

// Extend interfaces
interface NetworkDeviceDriver extends DeviceDriver {
  protocol: NetworkProtocol;
  openConnection(endpoint: NetworkEndpoint): Promise<NetworkConnection>;
  closeConnection(connectionId: string): Promise<void>;
}

// Use type aliases for unions, primitives, and complex types
type MemorySize = number;
type PermissionFlags = number;
type DeviceState = 'idle' | 'busy' | 'error' | 'disabled';

type HandlerFunction = (data: any) => void | Promise<void>;
type FilterFunction<T> = (item: T) => boolean;
type TransformFunction<T, U> = (item: T) => U;
```

### Generic Constraints

```typescript
// Basic generic with constraints
function createMemoryPool<T extends MemoryRegion>(regions: T[]): MemoryPool<T> {
  return new MemoryPool(regions);
}

// Multiple generic constraints
function mergeData<T extends { id: string }, U extends { id: string }>(
  list1: T[],
  list2: U[]
): Array<T & U> {
  // Implementation
  return [];
}

// Generic interface
interface Repository<T, ID = string> {
  findById(id: ID): Promise<T | null>;
  findAll(): Promise<T[]>;
  save(entity: T): Promise<T>;
  delete(id: ID): Promise<void>;
}

// Generic class with constraints
class MemoryAnalyzer<T extends MemoryRegion = MemoryRegion> {
  constructor(private regions: readonly T[]) {}
  
  public async analyze(): Promise<AnalysisResult<T>> {
    // Implementation
    return {} as AnalysisResult<T>;
  }
}
```

## ⚠️ Error Handling

### Custom Error Classes

```typescript
// Base error class
export class MultiOSError extends Error {
  public readonly timestamp: number;
  public readonly code?: string;
  
  constructor(message: string, code?: string) {
    super(message);
    this.name = this.constructor.name;
    this.timestamp = Date.now();
    this.code = code;
    
    // Maintains proper stack trace for where our error was thrown
    if (Error.captureStackTrace) {
      Error.captureStackTrace(this, this.constructor);
    }
  }
}

// Specific error types
export class MemoryError extends MultiOSError {
  constructor(message: string, public readonly address?: number) {
    super(message, 'MEMORY_ERROR');
  }
}

export class DeviceError extends MultiOSError {
  constructor(message: string, public readonly deviceId: string) {
    super(message, 'DEVICE_ERROR');
  }
}

export class NetworkError extends MultiOSError {
  constructor(
    message: string,
    public readonly protocol: NetworkProtocol,
    public readonly endpoint?: NetworkEndpoint
  ) {
    super(message, 'NETWORK_ERROR');
  }
}
```

### Error Handling Patterns

```typescript
// Try-catch with proper typing
async function allocateMemory(size: number, alignment: number = 1): Promise<MemoryAllocation> {
  try {
    validateAllocationParameters(size, alignment);
    
    const allocation = await memoryService.allocate(size, alignment);
    
    if (!allocation) {
      throw new MemoryError(`Failed to allocate ${size} bytes with alignment ${alignment}`);
    }
    
    return allocation;
  } catch (error) {
    if (error instanceof MemoryError) {
      // Re-throw known errors
      throw error;
    }
    
    if (error instanceof ValidationError) {
      throw new MemoryError(`Invalid parameters: ${error.message}`);
    }
    
    // Wrap unknown errors
    throw new MemoryError(`Unexpected error during allocation: ${error.message}`);
  }
}

// Error boundary pattern for React components
class ErrorBoundary extends React.Component<
  { children: React.ReactNode; fallback?: React.ComponentType<any> },
  { hasError: boolean; error?: Error }
> {
  constructor(props: any) {
    super(props);
    this.state = { hasError: false };
  }
  
  static getDerivedStateFromError(error: Error) {
    return { hasError: true, error };
  }
  
  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error('Error caught by boundary:', error, errorInfo);
  }
  
  render() {
    if (this.state.hasError && this.state.error) {
      const FallbackComponent = this.props.fallback || DefaultErrorFallback;
      return <FallbackComponent error={this.state.error} />;
    }
    
    return this.props.children;
  }
}

// Result pattern for safer error handling
type Result<T, E = Error> = 
  | { success: true; value: T }
  | { success: false; error: E };

function parseMemoryConfig(configJson: string): Result<MemoryConfig, ParseError> {
  try {
    const config = JSON.parse(configJson);
    const validated = validateMemoryConfig(config);
    
    if (!validated.success) {
      return { success: false, error: validated.error };
    }
    
    return { success: true, value: validated.value };
  } catch (error) {
    return { success: false, error: new ParseError(`Invalid JSON: ${error.message}`) };
  }
}
```

### Defensive Programming

```typescript
// Input validation
function validateMemoryRegion(region: any): region is MemoryRegion {
  return (
    typeof region === 'object' &&
    typeof region.address === 'number' &&
    typeof region.size === 'number' &&
    typeof region.permissions === 'number'
  );
}

function safeGetMemoryRegion(regions: MemoryRegion[], index: number): MemoryRegion | null {
  if (!Array.isArray(regions) || index < 0 || index >= regions.length) {
    return null;
  }
  
  const region = regions[index];
  return validateMemoryRegion(region) ? region : null;
}

// Nullish coalescing and optional chaining
function getMemoryRegionName(region: MemoryRegion): string {
  return region.name ?? `Region 0x${region.address.toString(16)}`;
}

function getDeviceStatus(device: Device | null): string {
  return device?.status ?? 'unknown';
}

// Type guards
function isNetworkPacket(obj: any): obj is NetworkPacket {
  return (
    obj &&
    typeof obj.source === 'object' &&
    typeof obj.destination === 'object' &&
    typeof obj.protocol === 'string' &&
    obj.data instanceof Uint8Array
  );
}

// Defensive API calls
async function fetchMemoryData(endpoint: string): Promise<MemoryData> {
  try {
    const response = await fetch(endpoint, {
      headers: {
        'Content-Type': 'application/json',
      },
    });
    
    if (!response.ok) {
      throw new NetworkError(`HTTP ${response.status}: ${response.statusText}`);
    }
    
    const data = await response.json();
    
    if (!isValidMemoryData(data)) {
      throw new ValidationError('Invalid memory data format');
    }
    
    return data;
  } catch (error) {
    if (error instanceof NetworkError || error instanceof ValidationError) {
      throw error;
    }
    
    // Handle network errors
    if (error instanceof TypeError && error.message.includes('fetch')) {
      throw new NetworkError('Network request failed');
    }
    
    throw new MultiOSError(`Unexpected error: ${error.message}`);
  }
}
```

## ⚡ Performance Guidelines

### Memory Efficiency

```typescript
// Use efficient data structures
class MemoryPool<T> {
  private readonly available: T[] = [];
  private readonly inUse = new Set<T>();
  
  constructor(
    private readonly factory: () => T,
    private readonly reset: (item: T) => void,
    private readonly maxSize: number = 100
  ) {}
  
  public acquire(): T {
    if (this.available.length === 0) {
      if (this.inUse.size >= this.maxSize) {
        throw new MemoryError('Pool exhausted');
      }
      return this.factory();
    }
    
    const item = this.available.pop()!;
    this.inUse.add(item);
    return item;
  }
  
  public release(item: T): void {
    if (!this.inUse.has(item)) {
      return; // Item not from this pool
    }
    
    this.inUse.delete(item);
    this.reset(item);
    this.available.push(item);
  }
}

// Avoid unnecessary allocations
function formatMemoryAddress(address: number, useCache: boolean = true): string {
  if (useCache) {
    // Cache formatted addresses
    return formatAddressCache.get(address) ?? formatAndCache(address);
  }
  
  return `0x${address.toString(16).toUpperCase().padStart(8, '0')}`;
}

// Use WeakMap for object-based caches
const memoryRegionCache = new WeakMap<MemoryRegion, RegionAnalysis>();

function analyzeMemoryRegion(region: MemoryRegion): RegionAnalysis {
  const cached = memoryRegionCache.get(region);
  if (cached) {
    return cached;
  }
  
  const analysis = performAnalysis(region);
  memoryRegionCache.set(region, analysis);
  return analysis;
}

// Object pooling for frequent allocations
class BufferPool {
  private readonly pool: Uint8Array[] = [];
  
  public get(size: number): Uint8Array {
    // Find suitable buffer or create new one
    const buffer = this.pool.find(buf => buf.length === size);
    
    if (buffer) {
      this.pool.splice(this.pool.indexOf(buffer), 1);
      return buffer;
    }
    
    return new Uint8Array(size);
  }
  
  public release(buffer: Uint8Array): void {
    // Reset buffer content
    buffer.fill(0);
    this.pool.push(buffer);
  }
}
```

### Event Handling Performance

```typescript
// Debounce and throttle for expensive operations
const debouncedAnalyzeMemory = useMemo(
  () => debounce(analyzeMemory, 300),
  [analyzeMemory]
);

const throttledUpdateView = useMemo(
  () => throttle(updateVisualization, 16), // 60fps
  [updateVisualization]
);

// Use event delegation
document.addEventListener('click', (event) => {
  const target = event.target as HTMLElement;
  
  if (target.matches('.memory-region')) {
    handleMemoryRegionClick(target);
  } else if (target.matches('.memory-region-controls')) {
    handleRegionControlClick(target);
  }
});

// Memoize expensive computations
const memoryStats = useMemo(() => {
  return calculateMemoryStatistics(regions);
}, [regions]);

// Use React.memo for expensive components
const MemoryRegionList = React.memo<MemoryRegionListProps>(({ regions }) => {
  return (
    <div className="memory-region-list">
      {regions.map(region => (
        <MemoryRegionComponent key={region.id} region={region} />
      ))}
    </div>
  );
});

const MemoryRegionComponent = React.memo<MemoryRegionComponentProps>(({ region }) => {
  // Component implementation
});
```

### Web Workers for Heavy Computation

```typescript
// Web worker for memory analysis
// memory-analyzer.worker.ts
self.addEventListener('message', (event: MessageEvent) => {
  const { type, data } = event.data;
  
  switch (type) {
    case 'ANALYZE_MEMORY':
      analyzeMemoryInWorker(data.regions)
        .then(result => {
          self.postMessage({
            type: 'ANALYSIS_COMPLETE',
            data: result
          });
        })
        .catch(error => {
          self.postMessage({
            type: 'ANALYSIS_ERROR',
            error: error.message
          });
        });
      break;
  }
});

async function analyzeMemoryInWorker(regions: MemoryRegion[]): Promise<AnalysisResult> {
  // Heavy computation in worker thread
  return performComplexAnalysis(regions);
}

// Main thread usage
class MemoryAnalyzerService {
  private worker: Worker;
  
  constructor() {
    this.worker = new Worker(new URL('./memory-analyzer.worker.ts', import.meta.url));
  }
  
  public analyzeMemory(regions: MemoryRegion[]): Promise<AnalysisResult> {
    return new Promise((resolve, reject) => {
      const handleMessage = (event: MessageEvent) => {
        const { type, data, error } = event.data;
        
        switch (type) {
          case 'ANALYSIS_COMPLETE':
            this.worker.removeEventListener('message', handleMessage);
            resolve(data);
            break;
          case 'ANALYSIS_ERROR':
            this.worker.removeEventListener('message', handleMessage);
            reject(new Error(error));
            break;
        }
      };
      
      this.worker.addEventListener('message', handleMessage);
      this.worker.postMessage({
        type: 'ANALYZE_MEMORY',
        data: { regions }
      });
    });
  }
}
```

## 🧪 Testing Standards

### Unit Testing with Jest

```typescript
// memory-analyzer.test.ts
import { MemoryAnalyzer } from '../services/memory-analyzer';
import { MemoryRegion } from '../types/memory';

describe('MemoryAnalyzer', () => {
  let analyzer: MemoryAnalyzer;
  let mockRegions: MemoryRegion[];
  
  beforeEach(() => {
    analyzer = new MemoryAnalyzer();
    mockRegions = [
      {
        address: 0x1000,
        size: 4096,
        permissions: 0x7, // RWX
        name: 'kernel'
      },
      {
        address: 0x2000,
        size: 2048,
        permissions: 0x3, // RW
        name: 'user_data'
      }
    ];
  });
  
  describe('analyzeMemory', () => {
    it('should correctly calculate total memory size', async () => {
      const result = await analyzer.analyzeMemory(mockRegions);
      
      expect(result.totalSize).toBe(6144); // 4096 + 2048
      expect(result.usedSize).toBe(4096); // Only RWX regions
    });
    
    it('should handle empty memory regions', async () => {
      const result = await analyzer.analyzeMemory([]);
      
      expect(result.totalSize).toBe(0);
      expect(result.usedSize).toBe(0);
      expect(result.regions).toHaveLength(0);
    });
    
    it('should identify overlapping regions', async () => {
      const overlappingRegions = [
        ...mockRegions,
        {
          address: 0x1500, // Overlaps with first region
          size: 1024,
          permissions: 0x3,
          name: 'overlap'
        }
      ];
      
      const result = await analyzer.analyzeMemory(overlappingRegions);
      
      expect(result.overlappingRegions).toHaveLength(1);
      expect(result.warnings).toContain('Memory regions overlap detected');
    });
  });
  
  describe('calculateFragmentation', () => {
    it('should calculate fragmentation ratio correctly', () => {
      const analyzer = new MemoryAnalyzer();
      const regions = [
        { address: 0x1000, size: 1024, permissions: 0x1, name: 'region1' },
        { address: 0x2000, size: 1024, permissions: 0x1, name: 'region2' }
      ];
      
      const fragmentation = analyzer.calculateFragmentation(regions, 4096);
      
      expect(fragmentation.ratio).toBeCloseTo(0.5, 2); // 2048 free / 4096 total
      expect(fragmentation.gaps).toHaveLength(1);
    });
  });
});

// Mock service for testing
jest.mock('../services/memory-service', () => ({
  MemoryService: jest.fn().mockImplementation(() => ({
    getMemoryRegions: jest.fn().mockResolvedValue([]),
    allocateMemory: jest.fn().mockResolvedValue({ address: 0x1000, size: 1024 }),
  }))
}));

// Integration tests
describe('MemoryAnalyzer Integration', () => {
  it('should work end-to-end', async () => {
    const analyzer = new MemoryAnalyzer();
    
    const allocation = await analyzer.allocate(1024, 4);
    expect(allocation).not.toBeNull();
    
    if (allocation) {
      const written = await analyzer.write(allocation.address, new Uint8Array([1, 2, 3]));
      expect(written).toBe(true);
      
      const data = await analyzer.read(allocation.address, 3);
      expect(data).toEqual(new Uint8Array([1, 2, 3]));
    }
  });
});
```

### Component Testing

```typescript
// MemoryVisualizer.test.tsx
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { MemoryVisualizer } from '../components/MemoryVisualizer';
import { MemoryRegion } from '../types/memory';

const mockRegions: MemoryRegion[] = [
  { address: 0x1000, size: 1024, permissions: 0x7, name: 'kernel' },
  { address: 0x2000, size: 2048, permissions: 0x3, name: 'user_data' }
];

describe('MemoryVisualizer', () => {
  it('should render memory regions correctly', () => {
    render(<MemoryVisualizer memoryMap={mockRegions} />);
    
    expect(screen.getByText('Memory Regions Visualization')).toBeInTheDocument();
    expect(screen.getByText('kernel')).toBeInTheDocument();
    expect(screen.getByText('user_data')).toBeInTheDocument();
  });
  
  it('should handle region selection', async () => {
    const onRegionSelect = jest.fn();
    
    render(
      <MemoryVisualizer 
        memoryMap={mockRegions} 
        onRegionSelect={onRegionSelect} 
      />
    );
    
    const regionElement = screen.getByText('kernel');
    fireEvent.click(regionElement);
    
    await waitFor(() => {
      expect(onRegionSelect).toHaveBeenCalledWith(mockRegions[0]);
    });
  });
  
  it('should show loading state during analysis', async () => {
    render(<MemoryVisualizer memoryMap={mockRegions} />);
    
    // Initially loading
    expect(screen.getByText(/analyzing/i)).toBeInTheDocument();
    
    // Wait for analysis to complete
    await waitFor(() => {
      expect(screen.queryByText(/analyzing/i)).not.toBeInTheDocument();
    });
  });
});
```

### Performance Testing

```typescript
// performance.test.ts
import { measurePerformance } from '../utils/performance';

describe('MemoryAnalyzer Performance', () => {
  it('should handle large memory maps efficiently', async () => {
    const largeMemoryMap = generateLargeMemoryMap(10000);
    
    const startTime = performance.now();
    const result = await analyzer.analyzeMemory(largeMemoryMap);
    const endTime = performance.now();
    
    expect(endTime - startTime).toBeLessThan(1000); // Should complete within 1 second
    expect(result.totalSize).toBeGreaterThan(0);
  });
  
  it('should not cause memory leaks', () => {
    const initialMemory = process.memoryUsage().heapUsed;
    
    for (let i = 0; i < 1000; i++) {
      analyzer.analyzeMemory(mockRegions);
    }
    
    // Force garbage collection if available
    if (global.gc) {
      global.gc();
    }
    
    const finalMemory = process.memoryUsage().heapUsed;
    const memoryIncrease = finalMemory - initialMemory;
    
    // Memory increase should be minimal
    expect(memoryIncrease).toBeLessThan(10 * 1024 * 1024); // 10MB
  });
});

function generateLargeMemoryMap(size: number): MemoryRegion[] {
  const regions: MemoryRegion[] = [];
  
  for (let i = 0; i < size; i++) {
    regions.push({
      address: i * 4096,
      size: 1024 + (i % 2048),
      permissions: i % 4,
      name: `region_${i}`
    });
  }
  
  return regions;
}
```

## 📦 Package Management

### Package.json Configuration

```json
{
  "name": "multios-web-tools",
  "version": "1.0.0",
  "description": "MultiOS web-based development tools",
  "main": "dist/index.js",
  "types": "dist/index.d.ts",
  "scripts": {
    "build": "tsc && vite build",
    "dev": "vite",
    "preview": "vite preview",
    "test": "jest",
    "test:watch": "jest --watch",
    "test:coverage": "jest --coverage",
    "lint": "eslint src --ext .ts,.tsx",
    "lint:fix": "eslint src --ext .ts,.tsx --fix",
    "format": "prettier --write src/**/*.{ts,tsx,css,md}",
    "type-check": "tsc --noEmit",
    "analyze": "npm run build && npx vite-bundle-analyzer dist"
  },
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "react-router-dom": "^6.8.0",
    "framer-motion": "^9.0.0",
    "react-hot-toast": "^2.4.0",
    "lodash": "^4.17.21",
    "date-fns": "^2.29.0",
    "zustand": "^4.3.0"
  },
  "devDependencies": {
    "@types/react": "^18.0.0",
    "@types/react-dom": "^18.0.0",
    "@types/lodash": "^4.14.191",
    "@typescript-eslint/eslint-plugin": "^5.51.0",
    "@typescript-eslint/parser": "^5.51.0",
    "@vitejs/plugin-react": "^3.1.0",
    "eslint": "^8.34.0",
    "eslint-config-prettier": "^8.6.0",
    "eslint-plugin-import": "^2.27.5",
    "eslint-plugin-react": "^7.32.2",
    "eslint-plugin-react-hooks": "^4.6.0",
    "prettier": "^2.8.4",
    "typescript": "^4.9.5",
    "vite": "^4.1.0",
    "jest": "^29.4.3",
    "@testing-library/react": "^14.0.0",
    "@testing-library/jest-dom": "^5.16.5",
    "jest-environment-jsdom": "^29.4.3"
  },
  "engines": {
    "node": ">=16.0.0",
    "npm": ">=8.0.0"
  },
  "browserslist": [
    "> 1%",
    "last 2 versions",
    "not dead"
  ]
}
```

### Build Configuration

```typescript
// vite.config.ts
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { resolve } from 'path';

export default defineConfig({
  plugins: [react()],
  
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
      '@/components': resolve(__dirname, 'src/components'),
      '@/services': resolve(__dirname, 'src/services'),
      '@/types': resolve(__dirname, 'src/types'),
      '@/utils': resolve(__dirname, 'src/utils'),
    },
  },
  
  build: {
    target: 'es2015',
    outDir: 'dist',
    sourcemap: true,
    rollupOptions: {
      output: {
        manualChunks: {
          vendor: ['react', 'react-dom'],
          utils: ['lodash', 'date-fns'],
          router: ['react-router-dom'],
        },
      },
    },
  },
  
  server: {
    port: 3000,
    host: true,
    open: true,
  },
  
  optimizeDeps: {
    include: ['react', 'react-dom', 'react-router-dom'],
  },
});
```

### Module Structure

```
src/
├── components/
│   ├── common/
│   │   ├── Button.tsx
│   │   ├── LoadingSpinner.tsx
│   │   └── ErrorBoundary.tsx
│   ├── memory/
│   │   ├── MemoryVisualizer.tsx
│   │   ├── MemoryRegionList.tsx
│   │   └── MemoryStats.tsx
│   └── network/
│       ├── NetworkTopology.tsx
│       └── ProtocolAnalyzer.tsx
├── services/
│   ├── api/
│   │   ├── memory.ts
│   │   ├── network.ts
│   │   └── devices.ts
│   ├── analysis/
│   │   ├── memory-analyzer.ts
│   │   ├── network-analyzer.ts
│   │   └── performance.ts
│   └── realtime/
│       ├── event-stream.ts
│       └── websocket.ts
├── types/
│   ├── memory.ts
│   ├── network.ts
│   ├── devices.ts
│   └── api.ts
├── hooks/
│   ├── useMemory.ts
│   ├── useNetwork.ts
│   └── useRealTime.ts
├── utils/
│   ├── formatters.ts
│   ├── validation.ts
│   └── performance.ts
└── styles/
    ├── globals.css
    ├── components.css
    └── themes/
        ├── dark.ts
        └── light.ts
```

---

## 📋 Checklist for Code Reviews

When reviewing JavaScript/TypeScript code, ensure:

- [ ] Code follows ESLint and Prettier formatting rules
- [ ] TypeScript types are properly defined and used
- [ ] No `any` types without explicit justification
- [ ] Component props are properly typed
- [ ] Error handling is comprehensive and user-friendly
- [ ] Performance considerations for large data sets
- [ ] Tests are included and have good coverage
- [ ] Accessibility standards are followed
- [ ] Bundle size is optimized (use code splitting)
- [ ] Dependencies are minimal and up-to-date
- [ ] Security best practices are followed (no eval, XSS prevention)

*Last Updated: November 3, 2025*
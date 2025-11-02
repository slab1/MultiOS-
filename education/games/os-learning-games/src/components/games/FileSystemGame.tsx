import React, { useState, useCallback, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import { Folder, File, Plus, Trash2, Search, HardDrive, FolderOpen } from 'lucide-react';

interface FileSystemNode {
  id: string;
  name: string;
  type: 'file' | 'directory';
  size: number;
  parentId: string | null;
  created: Date;
  permissions: {
    read: boolean;
    write: boolean;
    execute: boolean;
  };
  children?: string[];
}

interface FileSystemGameProps {
  onComplete: (score: number, metrics: any) => void;
  difficulty: 'beginner' | 'intermediate' | 'advanced';
}

export const FileSystemGame: React.FC<FileSystemGameProps> = ({ onComplete, difficulty }) => {
  const [fileSystem, setFileSystem] = useState<Map<string, FileSystemNode>>(new Map());
  const [currentPath, setCurrentPath] = useState<string>('root');
  const [selectedItem, setSelectedItem] = useState<string | null>(null);
  const [gameMode, setGameMode] = useState<'navigate' | 'organize' | 'backup' | 'optimize'>('navigate');
  const [score, setScore] = useState(100);
  const [moves, setMoves] = useState(0);
  const [gameState, setGameState] = useState<'playing' | 'completed'>('playing');
  
  // File system metrics
  const [metrics, setMetrics] = useState({
    totalFiles: 0,
    totalDirectories: 0,
    diskUsage: 0,
    maxDepth: 0,
    fragmentation: 0,
    searchEfficiency: 100,
  });

  const initializeFileSystem = useCallback(() => {
    const fs = new Map<string, FileSystemNode>();
    
    // Create root directory
    const root: FileSystemNode = {
      id: 'root',
      name: '/',
      type: 'directory',
      size: 0,
      parentId: null,
      created: new Date(),
      permissions: { read: true, write: true, execute: true },
      children: [],
    };
    
    fs.set('root', root);

    // Create initial file structure based on difficulty
    const createFiles = (count: number, parentId: string) => {
      for (let i = 0; i < count; i++) {
        const fileId = `file_${Date.now()}_${i}`;
        const dirId = `dir_${Date.now()}_${i}`;
        
        // Create file
        const file: FileSystemNode = {
          id: fileId,
          name: `document_${i + 1}.txt`,
          type: 'file',
          size: Math.floor(Math.random() * 1000) + 100, // 100-1100 bytes
          parentId,
          created: new Date(),
          permissions: { read: true, write: true, execute: false },
        };
        
        fs.set(fileId, file);
        
        // Add to parent children
        const parent = fs.get(parentId);
        if (parent) {
          parent.children = [...(parent.children || []), fileId];
        }
        
        // Create subdirectory occasionally
        if (Math.random() > 0.6 && difficulty !== 'beginner') {
          const subDir: FileSystemNode = {
            id: dirId,
            name: `folder_${i + 1}`,
            type: 'directory',
            size: 0,
            parentId,
            created: new Date(),
            permissions: { read: true, write: true, execute: true },
            children: [],
          };
          
          fs.set(dirId, subDir);
          if (parent) {
            parent.children = [...(parent.children || []), dirId];
          }
          
          // Add some files to subdirectory
          if (Math.random() > 0.3) {
            createFiles(Math.floor(Math.random() * 3) + 1, dirId);
          }
        }
      }
    };

    const initialDirectories = difficulty === 'beginner' ? 3 : difficulty === 'intermediate' ? 5 : 8;
    createFiles(initialDirectories, 'root');
    
    setFileSystem(fs);
    calculateMetrics(fs);
  }, [difficulty]);

  const calculateMetrics = (fs: Map<string, FileSystemNode>) => {
    let totalFiles = 0;
    let totalDirectories = 0;
    let maxDepth = 0;
    
    const calculateDepth = (nodeId: string, depth: number): number => {
      maxDepth = Math.max(maxDepth, depth);
      const node = fs.get(nodeId);
      if (!node || !node.children) return depth;
      
      return Math.max(...node.children.map(childId => calculateDepth(childId, depth + 1)));
    };

    fs.forEach(node => {
      if (node.type === 'file') {
        totalFiles++;
      } else {
        totalDirectories++;
      }
    });

    const diskUsage = Array.from(fs.values())
      .filter(node => node.type === 'file')
      .reduce((sum, node) => sum + node.size, 0);

    const depth = calculateDepth('root', 0);
    
    setMetrics({
      totalFiles,
      totalDirectories,
      diskUsage,
      maxDepth: depth,
      fragmentation: Math.random() * 20, // Simulated fragmentation
      searchEfficiency: Math.max(50, 100 - (totalFiles * 2) - (depth * 10)),
    });
  };

  useEffect(() => {
    initializeFileSystem();
  }, [initializeFileSystem]);

  const getCurrentDirectory = () => {
    return fileSystem.get(currentPath);
  };

  const getVisibleItems = () => {
    const currentDir = getCurrentDirectory();
    if (!currentDir || !currentDir.children) return [];
    
    return currentDir.children
      .map(childId => fileSystem.get(childId))
      .filter(Boolean)
      .sort((a, b) => {
        // Directories first, then files
        if (a!.type !== b!.type) {
          return a!.type === 'directory' ? -1 : 1;
        }
        return a!.name.localeCompare(b!.name);
      });
  };

  const navigateToDirectory = (dirId: string) => {
    const dir = fileSystem.get(dirId);
    if (dir && dir.type === 'directory') {
      setCurrentPath(dirId);
      setSelectedItem(null);
      setMoves(prev => prev + 1);
      setScore(prev => Math.max(prev - 1, 0));
    }
  };

  const createFile = (name: string, size: number = 512) => {
    const currentDir = getCurrentDirectory();
    if (!currentDir || !currentDir.children) return;

    const newFile: FileSystemNode = {
      id: `file_${Date.now()}`,
      name,
      type: 'file',
      size,
      parentId: currentPath,
      created: new Date(),
      permissions: { read: true, write: true, execute: false },
    };

    const updatedFs = new Map(fileSystem);
    updatedFs.set(newFile.id, newFile);
    
    const updatedDir = { ...currentDir };
    updatedDir.children = [...(updatedDir.children || []), newFile.id];
    updatedFs.set(currentPath, updatedDir);

    setFileSystem(updatedFs);
    calculateMetrics(updatedFs);
    setMoves(prev => prev + 1);
  };

  const createDirectory = (name: string) => {
    const currentDir = getCurrentDirectory();
    if (!currentDir || !currentDir.children) return;

    const newDir: FileSystemNode = {
      id: `dir_${Date.now()}`,
      name,
      type: 'directory',
      size: 0,
      parentId: currentPath,
      created: new Date(),
      permissions: { read: true, write: true, execute: true },
      children: [],
    };

    const updatedFs = new Map(fileSystem);
    updatedFs.set(newDir.id, newDir);
    
    const updatedDir = { ...currentDir };
    updatedDir.children = [...(updatedDir.children || []), newDir.id];
    updatedFs.set(currentPath, updatedDir);

    setFileSystem(updatedFs);
    setMoves(prev => prev + 1);
  };

  const deleteItem = (itemId: string) => {
    const item = fileSystem.get(itemId);
    if (!item) return;

    const updatedFs = new Map(fileSystem);
    
    // Remove from parent's children
    const parent = updatedFs.get(item.parentId!);
    if (parent && parent.children) {
      parent.children = parent.children.filter(id => id !== itemId);
      updatedFs.set(item.parentId!, parent);
    }

    // Remove the item and its children recursively
    const removeRecursive = (id: string) => {
      const node = updatedFs.get(id);
      if (!node) return;
      
      if (node.children) {
        node.children.forEach(childId => removeRecursive(childId));
      }
      
      updatedFs.delete(id);
    };

    removeRecursive(itemId);
    setFileSystem(updatedFs);
    calculateMetrics(updatedFs);
    setSelectedItem(null);
    setMoves(prev => prev + 1);
    setScore(prev => Math.max(prev - 5, 0));
  };

  const organizeFiles = () => {
    const currentDir = getCurrentDirectory();
    if (!currentDir || !currentDir.children) return;

    // Group files by type
    const files = currentDir.children
      .map(id => fileSystem.get(id))
      .filter(node => node?.type === 'file');

    const directories = currentDir.children
      .map(id => fileSystem.get(id))
      .filter(node => node?.type === 'directory');

    // Create type-based directories
    const types = ['documents', 'images', 'videos', 'audio', 'archives'];
    const updatedFs = new Map(fileSystem);
    
    types.forEach(type => {
      const typeDirId = `type_${type}_${Date.now()}`;
      const typeDir: FileSystemNode = {
        id: typeDirId,
        name: type,
        type: 'directory',
        size: 0,
        parentId: currentPath,
        created: new Date(),
        permissions: { read: true, write: true, execute: true },
        children: [],
      };
      
      updatedFs.set(typeDirId, typeDir);
    });

    // Move files to appropriate directories
    files.forEach(file => {
      if (!file) return;
      
      const extension = file.name.split('.').pop()?.toLowerCase();
      let targetDirName = 'documents';
      
      if (extension) {
        if (['jpg', 'jpeg', 'png', 'gif', 'bmp', 'svg'].includes(extension)) {
          targetDirName = 'images';
        } else if (['mp4', 'avi', 'mov', 'wmv', 'mkv'].includes(extension)) {
          targetDirName = 'videos';
        } else if (['mp3', 'wav', 'ogg', 'aac'].includes(extension)) {
          targetDirName = 'audio';
        } else if (['zip', 'rar', '7z', 'tar', 'gz'].includes(extension)) {
          targetDirName = 'archives';
        }
      }
      
      const targetDir = Array.from(updatedFs.values())
        .find(node => node.name === targetDirName && node.parentId === currentPath);
      
      if (targetDir) {
        // Move file
        const updatedFile = { ...file, parentId: targetDir.id };
        updatedFs.set(file.id, updatedFile);
        
        // Update parent directories
        const currentParent = updatedFs.get(currentPath)!;
        const targetParent = updatedFs.get(targetDir.id)!;
        
        currentParent.children = currentParent.children!.filter(id => id !== file.id);
        targetParent.children = [...(targetParent.children || []), file.id];
        
        updatedFs.set(currentPath, currentParent);
        updatedFs.set(targetDir.id, targetParent);
      }
    });

    setFileSystem(updatedFs);
    calculateMetrics(updatedFs);
    setMoves(prev => prev + 1);
    setScore(prev => Math.max(prev - 2, 0));
  };

  const searchFiles = (query: string) => {
    const results: FileSystemNode[] = [];
    
    fileSystem.forEach(node => {
      if (node.type === 'file' && node.name.toLowerCase().includes(query.toLowerCase())) {
        results.push(node);
      }
    });
    
    return results;
  };

  const getBreadcrumb = () => {
    const breadcrumb: string[] = [];
    let currentId = currentPath;
    
    while (currentId) {
      const node = fileSystem.get(currentId);
      if (node) {
        breadcrumb.unshift(node.name);
        currentId = node.parentId || '';
      } else {
        break;
      }
    }
    
    return breadcrumb;
  };

  const resetGame = () => {
    initializeFileSystem();
    setCurrentPath('root');
    setSelectedItem(null);
    setScore(100);
    setMoves(0);
    setGameState('playing');
  };

  const completeGame = () => {
    const finalScore = Math.max(score - moves * 2, 0);
    setGameState('completed');
    onComplete(finalScore, metrics);
  };

  const renderItem = (item: FileSystemNode) => (
    <div
      key={item.id}
      className={`p-3 rounded cursor-pointer transition-all ${
        selectedItem === item.id
          ? 'bg-blue-600 text-white'
          : 'bg-slate-700 hover:bg-slate-600 text-gray-200'
      }`}
      onClick={() => {
        setSelectedItem(item.id);
        if (item.type === 'directory') {
          navigateToDirectory(item.id);
        }
      }}
    >
      <div className="flex items-center gap-2">
        {item.type === 'directory' ? (
          <FolderOpen className="w-5 h-5" />
        ) : (
          <File className="w-5 h-5" />
        )}
        <div className="flex-1">
          <div className="font-medium">{item.name}</div>
          <div className="text-xs opacity-75">
            {item.type === 'file' ? `${item.size} bytes` : `${(item.children || []).length} items`}
          </div>
        </div>
      </div>
    </div>
  );

  if (gameState === 'completed') {
    return (
      <Card className="bg-gradient-to-r from-green-600 to-blue-600 border-0">
        <CardContent className="pt-6 text-center">
          <h3 className="text-2xl font-bold text-white mb-2">File System Challenge Complete!</h3>
          <div className="space-y-2 text-white/90">
            <div>Final Score: {Math.max(score - moves * 2, 0)}</div>
            <div>Total Files: {metrics.totalFiles}</div>
            <div>Total Directories: {metrics.totalDirectories}</div>
            <div>Disk Usage: {metrics.diskUsage} bytes</div>
            <div>Max Depth: {metrics.maxDepth}</div>
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-2xl font-bold text-white">File System Explorer</h2>
          <p className="text-gray-300">Navigate and organize file systems</p>
        </div>
        <div className="flex gap-2">
          <Badge variant="secondary">Score: {score}</Badge>
          <Badge variant="secondary">Moves: {moves}</Badge>
        </div>
      </div>

      {/* Game Mode Selection */}
      <Card className="bg-slate-800 border-slate-700">
        <CardHeader>
          <CardTitle className="text-white">Game Mode</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-2">
            {(['navigate', 'organize', 'backup', 'optimize'] as const).map((mode) => (
              <Button
                key={mode}
                variant={gameMode === mode ? 'default' : 'outline'}
                size="sm"
                onClick={() => setGameMode(mode)}
                className="capitalize"
              >
                {mode}
              </Button>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* File System View */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* File Explorer */}
        <div className="lg:col-span-2 space-y-4">
          {/* Breadcrumb */}
          <Card className="bg-slate-800 border-slate-700">
            <CardContent className="pt-6">
              <div className="flex items-center gap-2 text-white">
                {getBreadcrumb().map((crumb, index) => (
                  <React.Fragment key={index}>
                    {index > 0 && <span className="text-gray-400">/</span>}
                    <span 
                      className="cursor-pointer hover:text-blue-400"
                      onClick={() => {
                        const path = getBreadcrumb().slice(0, index + 1).join('/');
                        // Navigate to this level (simplified for demo)
                        if (index === 0) setCurrentPath('root');
                      }}
                    >
                      {crumb}
                    </span>
                  </React.Fragment>
                ))}
              </div>
            </CardContent>
          </Card>

          {/* File/Directory List */}
          <Card className="bg-slate-800 border-slate-700">
            <CardHeader className="flex flex-row items-center justify-between">
              <CardTitle className="text-white">
                Directory: {getCurrentDirectory()?.name}
              </CardTitle>
              <div className="flex gap-2">
                <Button size="sm" onClick={() => createFile(`new_file_${Date.now()}.txt`)}>
                  <Plus className="w-4 h-4" />
                </Button>
                <Button size="sm" onClick={() => createDirectory(`new_folder_${Date.now()}`)}>
                  <Folder className="w-4 h-4" />
                </Button>
              </div>
            </CardHeader>
            <CardContent>
              <div className="space-y-2">
                {getVisibleItems().map(item => renderItem(item!))}
                {getVisibleItems().length === 0 && (
                  <div className="text-center text-gray-400 py-8">
                    This directory is empty
                  </div>
                )}
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Sidebar */}
        <div className="space-y-4">
          {/* File System Metrics */}
          <Card className="bg-slate-800 border-slate-700">
            <CardHeader>
              <CardTitle className="text-white flex items-center gap-2">
                <HardDrive className="w-5 h-5" />
                System Stats
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-3">
              <div className="flex justify-between">
                <span className="text-gray-300">Files:</span>
                <span className="text-white">{metrics.totalFiles}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-300">Directories:</span>
                <span className="text-white">{metrics.totalDirectories}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-300">Disk Usage:</span>
                <span className="text-white">{metrics.diskUsage} bytes</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-300">Max Depth:</span>
                <span className="text-white">{metrics.maxDepth}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-300">Search Efficiency:</span>
                <span className="text-white">{metrics.searchEfficiency.toFixed(1)}%</span>
              </div>
            </CardContent>
          </Card>

          {/* Actions */}
          <Card className="bg-slate-800 border-slate-700">
            <CardHeader>
              <CardTitle className="text-white">Actions</CardTitle>
            </CardHeader>
            <CardContent className="space-y-2">
              {selectedItem && (
                <Button
                  variant="destructive"
                  size="sm"
                  onClick={() => deleteItem(selectedItem)}
                  className="w-full"
                >
                  <Trash2 className="w-4 h-4 mr-2" />
                  Delete Selected
                </Button>
              )}
              
              {gameMode === 'organize' && (
                <Button
                  variant="outline"
                  size="sm"
                  onClick={organizeFiles}
                  className="w-full"
                >
                  Organize by Type
                </Button>
              )}
              
              {gameMode === 'optimize' && (
                <Button
                  variant="outline"
                  size="sm"
                  onClick={completeGame}
                  className="w-full"
                >
                  Complete Optimization
                </Button>
              )}
              
              <Button
                variant="outline"
                size="sm"
                onClick={resetGame}
                className="w-full"
              >
                Reset System
              </Button>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  );
};
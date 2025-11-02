import React, { useEffect, useRef, useState } from 'react';
import * as d3 from 'd3';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { FileTree, Search, Folder, File, HardDrive, Users } from 'lucide-react';

interface FileSystemNode {
  id: string;
  name: string;
  type: 'directory' | 'file';
  size: number;
  inode: number;
  permissions: string;
  owner: string;
  group: string;
  modified: number;
  children?: FileSystemNode[];
  parent?: string;
  path: string;
  open?: boolean;
}

interface FileSystemVisualizationProps {
  realTimeData: boolean;
}

const FileSystemVisualization: React.FC<FileSystemVisualizationProps> = ({ realTimeData }) => {
  const svgRef = useRef<SVGSVGElement>(null);
  const [fileSystem, setFileSystem] = useState<FileSystemNode[]>([]);
  const [selectedNode, setSelectedNode] = useState<FileSystemNode | null>(null);
  const [searchTerm, setSearchTerm] = useState('');
  const [expandedNodes, setExpandedNodes] = useState<Set<string>>(new Set(['root']));
  const [viewMode, setViewMode] = useState<'tree' | 'inode'>('tree');

  // Generate realistic file system hierarchy
  const generateFileSystem = (): FileSystemNode[] => {
    const fileSystemNodes: FileSystemNode[] = [
      {
        id: 'root',
        name: '/',
        type: 'directory',
        size: 4096,
        inode: 2,
        permissions: 'drwxr-xr-x',
        owner: 'root',
        group: 'root',
        modified: Date.now() - 86400000,
        path: '/',
        open: true,
        children: []
      }
    ];

    // Root directories
    const rootDirectories = [
      { name: 'bin', type: 'directory' as const, size: 4096, inode: 3 },
      { name: 'boot', type: 'directory' as const, size: 4096, inode: 4 },
      { name: 'dev', type: 'directory' as const, size: 4096, inode: 5 },
      { name: 'etc', type: 'directory' as const, size: 4096, inode: 6 },
      { name: 'home', type: 'directory' as const, size: 4096, inode: 7 },
      { name: 'lib', type: 'directory' as const, size: 4096, inode: 8 },
      { name: 'media', type: 'directory' as const, size: 4096, inode: 9 },
      { name: 'mnt', type: 'directory' as const, size: 4096, inode: 10 },
      { name: 'opt', type: 'directory' as const, size: 4096, inode: 11 },
      { name: 'proc', type: 'directory' as const, size: 4096, inode: 12 },
      { name: 'root', type: 'directory' as const, size: 4096, inode: 13 },
      { name: 'run', type: 'directory' as const, size: 4096, inode: 14 },
      { name: 'sbin', type: 'directory' as const, size: 4096, inode: 15 },
      { name: 'srv', type: 'directory' as const, size: 4096, inode: 16 },
      { name: 'sys', type: 'directory' as const, size: 4096, inode: 17 },
      { name: 'tmp', type: 'directory' as const, size: 4096, inode: 18 },
      { name: 'usr', type: 'directory' as const, size: 4096, inode: 19 },
      { name: 'var', type: 'directory' as const, size: 4096, inode: 20 }
    ];

    rootDirectories.forEach((dir, index) => {
      const node: FileSystemNode = {
        id: `dir-${index}`,
        name: dir.name,
        type: dir.type,
        size: dir.size,
        inode: dir.inode,
        permissions: 'drwxr-xr-x',
        owner: 'root',
        group: 'root',
        modified: Date.now() - Math.random() * 86400000,
        parent: 'root',
        path: `/${dir.name}`,
        children: []
      };
      fileSystemNodes.push(node);
      fileSystemNodes[0].children!.push(node);
    });

    // Generate realistic files in each directory
    const generateFiles = (parent: FileSystemNode, depth: number) => {
      if (depth > 2) return; // Limit depth

      const numFiles = Math.floor(Math.random() * 20) + 5;
      const fileTypes = [
        { ext: 'txt', type: 'file' as const },
        { ext: 'conf', type: 'file' as const },
        { ext: 'log', type: 'file' as const },
        { ext: 'dat', type: 'file' as const },
        { ext: 'bin', type: 'file' as const },
        { ext: 'sh', type: 'file' as const },
        { ext: 'py', type: 'file' as const },
        { ext: 'js', type: 'file' as const },
        { ext: 'json', type: 'file' as const },
        { ext: 'xml', type: 'file' as const }
      ];

      const subdirs = ['config', 'lib', 'bin', 'doc', 'data', 'cache', 'temp'];

      for (let i = 0; i < numFiles; i++) {
        const fileType = fileTypes[Math.floor(Math.random() * fileTypes.length)];
        const isDirectory = Math.random() < 0.15 && depth < 2;
        
        if (isDirectory) {
          const subdirName = subdirs[Math.floor(Math.random() * subdirs.length)];
          const subdirNode: FileSystemNode = {
            id: `${parent.id}-subdir-${i}`,
            name: subdirName,
            type: 'directory',
            size: 4096,
            inode: Math.floor(Math.random() * 100000) + 1000,
            permissions: 'drwxr-xr-x',
            owner: Math.random() > 0.5 ? 'root' : 'user',
            group: 'users',
            modified: Date.now() - Math.random() * 86400000,
            parent: parent.id,
            path: `${parent.path}/${subdirName}`,
            children: []
          };
          parent.children!.push(subdirNode);
          generateFiles(subdirNode, depth + 1);
        } else {
          const fileNode: FileSystemNode = {
            id: `${parent.id}-file-${i}`,
            name: `${fileType.ext}-${String(i).padStart(3, '0')}.${fileType.ext}`,
            type: 'file',
            size: Math.floor(Math.random() * 1000000) + 1024, // 1KB to 1MB
            inode: Math.floor(Math.random() * 100000) + 1000,
            permissions: fileType.ext === 'conf' || fileType.ext === 'log' ? '-rw-r--r--' : '-rw-rw-r--',
            owner: Math.random() > 0.5 ? 'root' : 'user',
            group: 'users',
            modified: Date.now() - Math.random() * 86400000,
            parent: parent.id,
            path: `${parent.path}/${fileType.ext}-${String(i).padStart(3, '0')}.${fileType.ext}`
          };
          parent.children!.push(fileNode);
        }
      }
    };

    // Generate contents for each directory
    rootDirectories.forEach((dir, index) => {
      const dirNode = fileSystemNodes.find(n => n.id === `dir-${index}`);
      if (dirNode) {
        generateFiles(dirNode, 1);
      }
    });

    return fileSystemNodes;
  };

  // Find node by ID in the file system tree
  const findNode = (nodes: FileSystemNode[], id: string): FileSystemNode | null => {
    for (const node of nodes) {
      if (node.id === id) return node;
      if (node.children) {
        const found = findNode(node.children, id);
        if (found) return found;
      }
    }
    return null;
  };

  // Flatten file system for easy search
  const flattenNodes = (nodes: FileSystemNode[]): FileSystemNode[] => {
    const result: FileSystemNode[] = [];
    const traverse = (nodeList: FileSystemNode[]) => {
      nodeList.forEach(node => {
        result.push(node);
        if (node.children) {
          traverse(node.children);
        }
      });
    };
    traverse(nodes);
    return result;
  };

  // Filter nodes based on search term
  const filterNodes = (nodes: FileSystemNode[], searchTerm: string): FileSystemNode[] => {
    if (!searchTerm) return nodes;

    const filtered: FileSystemNode[] = [];
    const traverse = (nodeList: FileSystemNode[]) => {
      nodeList.forEach(node => {
        const matches = node.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
                       node.path.toLowerCase().includes(searchTerm.toLowerCase());
        
        if (matches) {
          filtered.push(node);
        }
        
        if (node.children) {
          traverse(node.children);
        }
      });
    };
    traverse(nodes);
    return filtered;
  };

  // Initialize file system
  useEffect(() => {
    setFileSystem(generateFileSystem());
  }, []);

  // Update file system for real-time changes
  useEffect(() => {
    if (!realTimeData) return;

    const interval = setInterval(() => {
      setFileSystem(prev => {
        const allNodes = flattenNodes(prev);
        const randomNode = allNodes[Math.floor(Math.random() * allNodes.length)];
        
        // Randomly update modification time to simulate activity
        if (randomNode && Math.random() > 0.8) {
          randomNode.modified = Date.now();
        }
        
        return [...prev];
      });
    }, 5000);

    return () => clearInterval(interval);
  }, [realTimeData]);

  // Render tree visualization
  useEffect(() => {
    if (!svgRef.current || fileSystem.length === 0) return;

    const svg = d3.select(svgRef.current);
    const width = 1000;
    const height = 600;

    svg.selectAll('*').remove();

    const g = svg.append('g')
      .attr('transform', 'translate(50,50)');

    // Create tree layout
    const tree = d3.tree<FileSystemNode>()
      .size([height - 100, width - 200]);

    // Prepare hierarchical data
    const hierarchyData = {
      ...fileSystem[0],
      children: expandNodes(fileSystem[0].children || [], new Set(['root']))
    };

    const root = d3.hierarchy(hierarchyData);
    const treeData = tree(root);

    // Create links
    const links = g.selectAll('.tree-link')
      .data(treeData.links())
      .enter()
      .append('path')
      .attr('class', 'tree-link')
      .attr('d', d3.linkHorizontal<any, any>()
        .x(d => d.y)
        .y(d => d.x)
      )
      .attr('stroke', '#374151')
      .attr('stroke-width', 1.5)
      .attr('fill', 'none');

    // Create nodes
    const nodes = g.selectAll('.tree-node')
      .data(treeData.descendants())
      .enter()
      .append('g')
      .attr('class', 'tree-node')
      .attr('transform', d => `translate(${d.y},${d.x})`)
      .style('cursor', 'pointer')
      .on('click', (event, d) => {
        setSelectedNode(d.data);
        if (d.data.type === 'directory') {
          toggleNode(d.data.id);
        }
      });

    // Add node icons
    nodes.append('text')
      .attr('x', -15)
      .attr('y', 0)
      .attr('text-anchor', 'middle')
      .attr('dominant-baseline', 'middle')
      .style('font-size', '16px')
      .style('fill', d => d.data.type === 'directory' ? '#3b82f6' : '#10b981')
      .text(d => d.data.type === 'directory' ? '📁' : '📄')
      .on('click', (event, d) => {
        event.stopPropagation();
        setSelectedNode(d.data);
        if (d.data.type === 'directory') {
          toggleNode(d.data.id);
        }
      });

    // Add node labels
    nodes.append('text')
      .attr('x', 5)
      .attr('y', 0)
      .attr('dominant-baseline', 'middle')
      .style('fill', '#f9fafb')
      .style('font-size', '11px')
      .text(d => {
        const name = d.data.name;
        return name.length > 20 ? name.substring(0, 17) + '...' : name;
      });

    // Add file size for files
    nodes.append('text')
      .attr('x', 5)
      .attr('y', 15)
      .attr('dominant-baseline', 'middle')
      .style('fill', '#9ca3af')
      .style('font-size', '9px')
      .text(d => {
        if (d.data.type === 'file') {
          return `${(d.data.size / 1024).toFixed(1)} KB`;
        }
        return '';
      });

  }, [fileSystem, expandedNodes]);

  // Expand nodes that should be visible
  const expandNodes = (nodes: FileSystemNode[], expanded: Set<string>): FileSystemNode[] => {
    return nodes.map(node => ({
      ...node,
      children: expanded.has(node.id) && node.children 
        ? expandNodes(node.children, expanded) 
        : undefined
    }));
  };

  // Toggle node expansion
  const toggleNode = (nodeId: string) => {
    setExpandedNodes(prev => {
      const newSet = new Set(prev);
      if (newSet.has(nodeId)) {
        newSet.delete(nodeId);
      } else {
        newSet.add(nodeId);
      }
      return newSet;
    });
  };

  // Calculate directory statistics
  const calculateStats = (nodes: FileSystemNode[]) => {
    let totalFiles = 0;
    let totalDirectories = 0;
    let totalSize = 0;
    const ownerCounts: Record<string, number> = {};

    const traverse = (nodeList: FileSystemNode[]) => {
      nodeList.forEach(node => {
        if (node.type === 'file') {
          totalFiles++;
          totalSize += node.size;
        } else {
          totalDirectories++;
        }
        
        ownerCounts[node.owner] = (ownerCounts[node.owner] || 0) + 1;
        
        if (node.children) {
          traverse(node.children);
        }
      });
    };

    traverse(nodes);
    return { totalFiles, totalDirectories, totalSize, ownerCounts };
  };

  const filteredNodes = filterNodes(fileSystem, searchTerm);
  const stats = calculateStats(fileSystem);
  const searchResults = flattenedNodes.length;

  return (
    <div className="space-y-4">
      {/* Controls */}
      <Card className="bg-gray-900 border-gray-700">
        <CardHeader>
          <CardTitle className="flex items-center">
            <FileTree className="h-5 w-5 mr-2" />
            File System Hierarchy Controls
          </CardTitle>
          <CardDescription>Browse and search file system with inode tracking</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex flex-wrap gap-4 items-center">
            <div className="flex items-center space-x-2">
              <Search className="h-4 w-4" />
              <Input
                type="text"
                placeholder="Search files and directories..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                className="bg-gray-800 border-gray-600 text-sm w-64"
              />
            </div>
            <div className="flex space-x-2">
              <Button
                variant={viewMode === 'tree' ? 'default' : 'outline'}
                size="sm"
                onClick={() => setViewMode('tree')}
              >
                Tree View
              </Button>
              <Button
                variant={viewMode === 'inode' ? 'default' : 'outline'}
                size="sm"
                onClick={() => setViewMode('inode')}
              >
                Inode View
              </Button>
            </div>
            <div className="flex space-x-2">
              <Badge variant="outline">
                {stats.totalFiles} files
              </Badge>
              <Badge variant="outline">
                {stats.totalDirectories} directories
              </Badge>
              <Badge variant="outline">
                {(stats.totalSize / 1024 / 1024 / 1024).toFixed(1)} GB total
              </Badge>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* File System Tree Visualization */}
      <Card className="bg-gray-900 border-gray-700">
        <CardHeader>
          <CardTitle>File System Hierarchy</CardTitle>
          <CardDescription>Interactive tree view of the file system structure</CardDescription>
        </CardHeader>
        <CardContent>
          <svg
            ref={svgRef}
            width={1000}
            height={600}
            className="border border-gray-600 rounded"
            style={{ background: '#1f2937' }}
          />
        </CardContent>
      </Card>

      {/* Selected Node Details and Statistics */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        {/* Node Details */}
        {selectedNode && (
          <Card className="bg-gray-900 border-gray-700">
            <CardHeader>
              <CardTitle className="flex items-center">
                {selectedNode.type === 'directory' ? <Folder className="h-5 w-5 mr-2" /> : <File className="h-5 w-5 mr-2" />}
                {selectedNode.name}
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-3">
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <div className="text-sm text-gray-400">Type</div>
                    <Badge className={selectedNode.type === 'directory' ? 'bg-blue-500' : 'bg-green-500'}>
                      {selectedNode.type}
                    </Badge>
                  </div>
                  <div>
                    <div className="text-sm text-gray-400">Inode</div>
                    <div className="font-mono">{selectedNode.inode}</div>
                  </div>
                </div>

                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <div className="text-sm text-gray-400">Size</div>
                    <div className="font-mono">
                      {selectedNode.type === 'file' 
                        ? `${(selectedNode.size / 1024).toFixed(1)} KB`
                        : '4.0 KB (directory)'
                      }
                    </div>
                  </div>
                  <div>
                    <div className="text-sm text-gray-400">Permissions</div>
                    <div className="font-mono text-xs">{selectedNode.permissions}</div>
                  </div>
                </div>

                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <div className="text-sm text-gray-400">Owner</div>
                    <div className="flex items-center">
                      <Users className="h-3 w-3 mr-1" />
                      {selectedNode.owner}
                    </div>
                  </div>
                  <div>
                    <div className="text-sm text-gray-400">Group</div>
                    <div>{selectedNode.group}</div>
                  </div>
                </div>

                <div>
                  <div className="text-sm text-gray-400">Path</div>
                  <div className="font-mono text-xs bg-gray-800 p-2 rounded">
                    {selectedNode.path}
                  </div>
                </div>

                <div>
                  <div className="text-sm text-gray-400">Last Modified</div>
                  <div>{new Date(selectedNode.modified).toLocaleString()}</div>
                </div>

                {selectedNode.type === 'directory' && selectedNode.children && (
                  <div>
                    <div className="text-sm text-gray-400">Contents</div>
                    <div className="text-sm">
                      {selectedNode.children.length} items
                      <div className="text-xs text-gray-500 mt-1">
                        {selectedNode.children.filter(c => c.type === 'directory').length} directories, {' '}
                        {selectedNode.children.filter(c => c.type === 'file').length} files
                      </div>
                    </div>
                  </div>
                )}
              </div>
            </CardContent>
          </Card>
        )}

        {/* File System Statistics */}
        <Card className="bg-gray-900 border-gray-700">
          <CardHeader>
            <CardTitle className="flex items-center">
              <HardDrive className="h-5 w-5 mr-2" />
              File System Statistics
            </CardTitle>
            <CardDescription>Overview of file system usage and distribution</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              {/* Storage Usage */}
              <div>
                <div className="text-sm text-gray-400 mb-2">Storage Distribution</div>
                <div className="space-y-2">
                  <div className="flex justify-between items-center">
                    <span className="text-sm">Total Files</span>
                    <span className="font-mono">{stats.totalFiles.toLocaleString()}</span>
                  </div>
                  <div className="flex justify-between items-center">
                    <span className="text-sm">Total Directories</span>
                    <span className="font-mono">{stats.totalDirectories.toLocaleString()}</span>
                  </div>
                  <div className="flex justify-between items-center">
                    <span className="text-sm">Total Size</span>
                    <span className="font-mono">{(stats.totalSize / 1024 / 1024 / 1024).toFixed(2)} GB</span>
                  </div>
                </div>
              </div>

              {/* File Type Distribution */}
              <div>
                <div className="text-sm text-gray-400 mb-2">File Type Distribution</div>
                <div className="space-y-2">
                  {Object.entries(
                    flattenNodes(fileSystem).reduce((acc, node) => {
                      if (node.type === 'file') {
                        const ext = node.name.split('.').pop() || 'unknown';
                        acc[ext] = (acc[ext] || 0) + 1;
                      }
                      return acc;
                    }, {} as Record<string, number>)
                  ).slice(0, 5).map(([ext, count]) => (
                    <div key={ext} className="flex justify-between items-center">
                      <span className="text-sm">.{ext}</span>
                      <div className="flex items-center space-x-2">
                        <span className="text-xs text-gray-400">{count}</span>
                        <div className="w-16 bg-gray-700 rounded-full h-2">
                          <div
                            className="bg-blue-500 h-2 rounded-full"
                            style={{ width: `${(count / stats.totalFiles) * 100}%` }}
                          />
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              </div>

              {/* Owner Distribution */}
              <div>
                <div className="text-sm text-gray-400 mb-2">File Ownership</div>
                <div className="space-y-2">
                  {Object.entries(stats.ownerCounts).map(([owner, count]) => (
                    <div key={owner} className="flex justify-between items-center">
                      <span className="text-sm">{owner}</span>
                      <span className="text-xs text-gray-400">{count} files</span>
                    </div>
                  ))}
                </div>
              </div>

              {/* Search Results */}
              {searchTerm && (
                <div className="pt-4 border-t border-gray-700">
                  <div className="text-sm text-gray-400">Search Results</div>
                  <div className="text-2xl font-bold">{searchResults}</div>
                  <div className="text-xs text-gray-400">matching items found</div>
                </div>
              )}
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
};

export default FileSystemVisualization;
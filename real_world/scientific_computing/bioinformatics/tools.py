"""
Computational Biology and Bioinformatics Tools for Scientific Computing Education
===============================================================================

This module provides educational implementations of bioinformatics algorithms:
- DNA/RNA sequence analysis and alignment
- Phylogenetic tree construction
- Protein structure analysis
- Molecular evolution algorithms
- Genome analysis tools

Author: Scientific Computing Education Team
"""

import numpy as np
import math
from typing import Dict, List, Tuple, Optional, Union
from collections import defaultdict, Counter
import random


class SequenceAnalysis:
    """DNA/RNA sequence analysis and manipulation tools."""
    
    COMPLEMENT = {
        'A': 'T', 'T': 'A', 'G': 'C', 'C': 'G',
        'a': 't', 't': 'a', 'g': 'c', 'c': 'g',
        'U': 'A', 'u': 'a'  # RNA complements
    }
    
    CODON_TABLE = {
        'TTT': 'F', 'TTC': 'F', 'TTA': 'L', 'TTG': 'L',
        'TCT': 'S', 'TCC': 'S', 'TCA': 'S', 'TCG': 'S',
        'TAT': 'Y', 'TAC': 'Y', 'TAA': '*', 'TAG': '*',
        'TGT': 'C', 'TGC': 'C', 'TGA': '*', 'TGG': 'W',
        'CTT': 'L', 'CTC': 'L', 'CTA': 'L', 'CTG': 'L',
        'CCT': 'P', 'CCC': 'P', 'CCA': 'P', 'CCG': 'P',
        'CAT': 'H', 'CAC': 'H', 'CAA': 'Q', 'CAG': 'Q',
        'CGT': 'R', 'CGC': 'R', 'CGA': 'R', 'CGG': 'R',
        'ATT': 'I', 'ATC': 'I', 'ATA': 'I', 'ATG': 'M',
        'ACT': 'T', 'ACC': 'T', 'ACA': 'T', 'ACG': 'T',
        'AAT': 'N', 'AAC': 'N', 'AAA': 'K', 'AAG': 'K',
        'AGT': 'S', 'AGC': 'S', 'AGA': 'R', 'AGG': 'R',
        'GTT': 'V', 'GTC': 'V', 'GTA': 'V', 'GTG': 'V',
        'GCT': 'A', 'GCC': 'A', 'GCA': 'A', 'GCG': 'A',
        'GAT': 'D', 'GAC': 'D', 'GAA': 'E', 'GAG': 'E',
        'GGT': 'G', 'GGC': 'G', 'GGA': 'G', 'GGG': 'G'
    }
    
    @staticmethod
    def gc_content(sequence: str) -> float:
        """
        Calculate GC content of DNA sequence.
        
        Args:
            sequence: DNA sequence string
            
        Returns:
            GC content as fraction (0-1)
        """
        sequence = sequence.upper()
        gc_count = sequence.count('G') + sequence.count('C')
        total_count = len(sequence)
        
        return gc_count / total_count if total_count > 0 else 0
    
    @staticmethod
    def reverse_complement(sequence: str) -> str:
        """
        Generate reverse complement of DNA sequence.
        
        Args:
            sequence: DNA sequence string
            
        Returns:
            Reverse complement sequence
        """
        sequence = sequence.upper()
        complement = ''.join([SequenceAnalysis.COMPLEMENT.get(base, 'N') 
                             for base in reversed(sequence)])
        return complement
    
    @staticmethod
    def transcribe(dna_sequence: str) -> str:
        """
        Transcribe DNA to RNA (replace T with U).
        
        Args:
            dna_sequence: DNA sequence string
            
        Returns:
            RNA sequence string
        """
        return dna_sequence.upper().replace('T', 'U')
    
    @staticmethod
    def translate(rna_sequence: str, frame: int = 0) -> str:
        """
        Translate RNA sequence to protein.
        
        Args:
            rna_sequence: RNA sequence string
            frame: Reading frame (0, 1, or 2)
            
        Returns:
            Protein sequence string
        """
        rna_sequence = rna_sequence.upper().replace('T', 'U')
        protein = []
        
        for i in range(frame, len(rna_sequence) - 2, 3):
            codon = rna_sequence[i:i+3]
            if len(codon) == 3 and codon in SequenceAnalysis.CODON_TABLE:
                amino_acid = SequenceAnalysis.CODON_TABLE[codon]
                if amino_acid != '*':  # Stop codon
                    protein.append(amino_acid)
        
        return ''.join(protein)
    
    @staticmethod
    def find_orfs(sequence: str, min_length: int = 100) -> List[Dict]:
        """
        Find Open Reading Frames (ORFs) in DNA sequence.
        
        Args:
            sequence: DNA sequence string
            min_length: Minimum ORF length in nucleotides
            
        Returns:
            List of ORFs with start/end positions and frames
        """
        sequence = sequence.upper()
        orfs = []
        
        # Find start and stop codons
        start_codon = 'ATG'
        stop_codons = ['TAA', 'TAG', 'TGA']
        
        for frame in range(3):
            # Forward strand
            for i in range(frame, len(sequence) - 2, 3):
                if sequence[i:i+3] == start_codon:
                    # Look for stop codon
                    for j in range(i + 3, len(sequence) - 2, 3):
                        if sequence[j:j+3] in stop_codons:
                            orf_length = j - i + 3
                            if orf_length >= min_length:
                                orfs.append({
                                    'start': i,
                                    'end': j + 3,
                                    'frame': frame,
                                    'strand': '+',
                                    'length': orf_length,
                                    'sequence': sequence[i:j+3]
                                })
                            break
            
            # Reverse strand (complement)
            rev_comp = SequenceAnalysis.reverse_complement(sequence)
            for i in range(frame, len(rev_comp) - 2, 3):
                if rev_comp[i:i+3] == start_codon:
                    for j in range(i + 3, len(rev_comp) - 2, 3):
                        if rev_comp[j:j+3] in stop_codons:
                            orf_length = j - i + 3
                            if orf_length >= min_length:
                                # Convert back to original coordinates
                                start_pos = len(sequence) - (j + 3)
                                end_pos = len(sequence) - i
                                orfs.append({
                                    'start': start_pos,
                                    'end': end_pos,
                                    'frame': frame,
                                    'strand': '-',
                                    'length': orf_length,
                                    'sequence': sequence[start_pos:end_pos]
                                })
                            break
        
        return sorted(orfs, key=lambda x: x['length'], reverse=True)
    
    @staticmethod
    def motif_finding(sequence: str, motif: str, positions_only: bool = False) -> Union[List[int], List[Dict]]:
        """
        Find all occurrences of a motif in a sequence.
        
        Args:
            sequence: Main sequence
            motif: Motif to search for
            positions_only: If True, return only positions; if False, return detailed info
            
        Returns:
            List of positions or detailed information about matches
        """
        sequence = sequence.upper()
        motif = motif.upper()
        positions = []
        
        for i in range(len(sequence) - len(motif) + 1):
            if sequence[i:i+len(motif)] == motif:
                positions.append(i)
        
        if positions_only:
            return positions
        else:
            return [{'position': pos, 'sequence': motif} for pos in positions]


class SequenceAlignment:
    """Global and local sequence alignment algorithms."""
    
    @staticmethod
    def needleman_wunsch(seq1: str, seq2: str, match_score: int = 1, 
                        mismatch_score: int = -1, gap_score: int = -2) -> Dict:
        """
        Perform global sequence alignment using Needleman-Wunsch algorithm.
        
        Args:
            seq1: First sequence
            seq2: Second sequence
            match_score: Score for matching characters
            mismatch_score: Score for mismatching characters
            gap_score: Score for gaps
            
        Returns:
            Dictionary with alignment results
        """
        m, n = len(seq1), len(seq2)
        
        # Initialize scoring matrix
        score_matrix = np.zeros((m + 1, n + 1))
        
        # Initialize first row and column
        for i in range(1, m + 1):
            score_matrix[i][0] = i * gap_score
        for j in range(1, n + 1):
            score_matrix[0][j] = j * gap_score
        
        # Fill scoring matrix
        for i in range(1, m + 1):
            for j in range(1, n + 1):
                if seq1[i-1] == seq2[j-1]:
                    match = score_matrix[i-1][j-1] + match_score
                else:
                    match = score_matrix[i-1][j-1] + mismatch_score
                
                delete = score_matrix[i-1][j] + gap_score
                insert = score_matrix[i][j-1] + gap_score
                
                score_matrix[i][j] = max(match, delete, insert)
        
        # Traceback
        aligned1, aligned2 = [], []
        i, j = m, n
        
        while i > 0 or j > 0:
            current_score = score_matrix[i][j]
            
            if i > 0 and j > 0:
                if seq1[i-1] == seq2[j-1]:
                    diagonal_score = score_matrix[i-1][j-1] + match_score
                else:
                    diagonal_score = score_matrix[i-1][j-1] + mismatch_score
                
                if current_score == diagonal_score:
                    aligned1.append(seq1[i-1])
                    aligned2.append(seq2[j-1])
                    i -= 1
                    j -= 1
                elif current_score == score_matrix[i-1][j] + gap_score:
                    aligned1.append(seq1[i-1])
                    aligned2.append('-')
                    i -= 1
                else:
                    aligned1.append('-')
                    aligned2.append(seq2[j-1])
                    j -= 1
            elif i > 0:
                aligned1.append(seq1[i-1])
                aligned2.append('-')
                i -= 1
            else:
                aligned1.append('-')
                aligned2.append(seq2[j-1])
                j -= 1
        
        return {
            'score': score_matrix[m][n],
            'aligned_seq1': ''.join(reversed(aligned1)),
            'aligned_seq2': ''.join(reversed(aligned2)),
            'score_matrix': score_matrix
        }
    
    @staticmethod
    def smith_waterman(seq1: str, seq2: str, match_score: int = 2,
                      mismatch_score: int = -1, gap_score: int = -2) -> Dict:
        """
        Perform local sequence alignment using Smith-Waterman algorithm.
        
        Args:
            seq1: First sequence
            seq2: Second sequence
            match_score: Score for matching characters
            mismatch_score: Score for mismatching characters
            gap_score: Score for gaps
            
        Returns:
            Dictionary with alignment results
        """
        m, n = len(seq1), len(seq2)
        
        # Initialize scoring matrix
        score_matrix = np.zeros((m + 1, n + 1))
        
        # Track max score and position
        max_score = 0
        max_i, max_j = 0, 0
        
        # Fill scoring matrix
        for i in range(1, m + 1):
            for j in range(1, n + 1):
                if seq1[i-1] == seq2[j-1]:
                    diagonal = score_matrix[i-1][j-1] + match_score
                else:
                    diagonal = score_matrix[i-1][j-1] + mismatch_score
                
                delete = score_matrix[i-1][j] + gap_score
                insert = score_matrix[i][j-1] + gap_score
                
                score_matrix[i][j] = max(0, diagonal, delete, insert)
                
                if score_matrix[i][j] > max_score:
                    max_score = score_matrix[i][j]
                    max_i, max_j = i, j
        
        # Traceback from max score
        aligned1, aligned2 = [], []
        i, j = max_i, max_j
        
        while i > 0 and j > 0 and score_matrix[i][j] > 0:
            current_score = score_matrix[i][j]
            
            if seq1[i-1] == seq2[j-1]:
                diagonal_score = score_matrix[i-1][j-1] + match_score
            else:
                diagonal_score = score_matrix[i-1][j-1] + mismatch_score
            
            if current_score == diagonal_score:
                aligned1.append(seq1[i-1])
                aligned2.append(seq2[j-1])
                i -= 1
                j -= 1
            elif current_score == score_matrix[i-1][j] + gap_score:
                aligned1.append(seq1[i-1])
                aligned2.append('-')
                i -= 1
            else:
                aligned1.append('-')
                aligned2.append(seq2[j-1])
                j -= 1
        
        return {
            'score': max_score,
            'aligned_seq1': ''.join(reversed(aligned1)),
            'aligned_seq2': ''.join(reversed(aligned2)),
            'score_matrix': score_matrix,
            'start_position': (i + 1, j + 1),
            'end_position': (max_i, max_j)
        }


class Phylogenetics:
    """Phylogenetic tree construction and analysis."""
    
    @staticmethod
    def calculate_distance_matrix(sequences: List[str], method: str = 'hamming') -> np.ndarray:
        """
        Calculate pairwise distance matrix between sequences.
        
        Args:
            sequences: List of DNA/protein sequences
            method: Distance calculation method
            
        Returns:
            Distance matrix
        """
        n = len(sequences)
        distances = np.zeros((n, n))
        
        for i in range(n):
            for j in range(i + 1, n):
                if method == 'hamming':
                    dist = Phylogenetics._hamming_distance(sequences[i], sequences[j])
                elif method == 'p_distance':
                    dist = Phylogenetics._p_distance(sequences[i], sequences[j])
                else:
                    raise ValueError(f"Unknown distance method: {method}")
                
                distances[i, j] = distances[j, i] = dist
        
        return distances
    
    @staticmethod
    def _hamming_distance(seq1: str, seq2: str) -> int:
        """Calculate Hamming distance between two sequences."""
        if len(seq1) != len(seq2):
            raise ValueError("Sequences must be same length for Hamming distance")
        return sum(c1 != c2 for c1, c2 in zip(seq1, seq2))
    
    @staticmethod
    def _p_distance(seq1: str, seq2: str) -> float:
        """Calculate p-distance (proportion of different sites)."""
        if len(seq1) != len(seq2):
            raise ValueError("Sequences must be same length for p-distance")
        
        differences = sum(c1 != c2 for c1, c2 in zip(seq1, seq2))
        return differences / len(seq1)
    
    @staticmethod
    def neighbor_joining(distance_matrix: np.ndarray, labels: List[str]) -> Dict:
        """
        Construct phylogenetic tree using Neighbor Joining algorithm.
        
        Args:
            distance_matrix: Pairwise distance matrix
            labels: Sequence labels
            
        Returns:
            Dictionary representing the phylogenetic tree
        """
        n = len(labels)
        
        # Active nodes (initial sequences)
        active_labels = labels.copy()
        active_indices = list(range(n))
        
        # Initialize tree structure
        tree = {}
        
        step = 0
        while len(active_indices) > 2:
            print(f"Neighbor Joining Step {step + 1}: {len(active_indices)} taxa remaining")
            
            # Calculate Q matrix
            r = np.sum(distance_matrix[active_indices][:, active_indices], axis=1)
            Q = np.zeros((len(active_indices), len(active_indices)))
            
            for i in range(len(active_indices)):
                for j in range(len(active_indices)):
                    if i != j:
                        Q[i, j] = (len(active_indices) - 2) * distance_matrix[active_indices[i], active_indices[j]] - r[i] - r[j]
            
            # Find minimum off-diagonal element
            min_i, min_j = np.unravel_index(np.argmin(Q + np.eye(len(active_indices)) * np.inf), Q.shape)
            
            # Calculate branch lengths
            d_ij = distance_matrix[active_indices[min_i], active_indices[min_j]]
            L_i = 0.5 * d_ij + 0.5 * (r[min_i] - r[min_j]) / (len(active_indices) - 2)
            L_j = d_ij - L_i
            
            # Create new node
            new_node = f"Node_{step}"
            new_label = f"N{step}"
            
            # Add to tree
            tree[new_node] = {
                'left': {'node': active_labels[min_i], 'distance': L_i},
                'right': {'node': active_labels[min_j], 'distance': L_j}
            }
            
            # Calculate distances to new node
            new_distances = np.zeros(len(active_indices))
            for k in range(len(active_indices)):
                if k not in [min_i, min_j]:
                    d_ik = distance_matrix[active_indices[min_i], active_indices[k]]
                    d_jk = distance_matrix[active_indices[min_j], active_indices[k]]
                    new_distances[k] = 0.5 * (d_ik + d_jk - d_ij)
            
            # Update distance matrix
            new_distance_matrix = np.zeros((len(active_indices) - 1, len(active_indices) - 1))
            
            # Copy existing distances, excluding min_i and min_j
            remaining_indices = [i for i in range(len(active_indices)) if i not in [min_i, min_j]]
            
            for i, old_i in enumerate(remaining_indices):
                for j, old_j in enumerate(remaining_indices):
                    new_distance_matrix[i, j] = distance_matrix[old_i, old_j]
            
            # Add distances to new node
            for i, old_i in enumerate(remaining_indices):
                new_distance_matrix[i, -1] = new_distances[old_i]
                new_distance_matrix[-1, i] = new_distances[old_i]
            
            distance_matrix = new_distance_matrix
            
            # Update active lists
            new_label_idx = len(active_labels)
            active_labels.append(new_label)
            active_indices.append(new_label_idx)
            
            # Remove min_i and min_j
            for idx in sorted([min_i, min_j], reverse=True):
                active_labels.pop(idx)
                active_indices.pop(idx)
            
            step += 1
        
        # Handle final connection
        if len(active_indices) == 2:
            final_distance = distance_matrix[0, 1]
            tree[f"Final_{step}"] = {
                'left': {'node': active_labels[0], 'distance': final_distance / 2},
                'right': {'node': active_labels[1], 'distance': final_distance / 2}
            }
        
        return tree
    
    @staticmethod
    def print_tree(tree: Dict, node: str = None, indent: str = "") -> None:
        """
        Print phylogenetic tree in Newick-like format.
        
        Args:
            tree: Tree dictionary
            node: Current node (None for root)
            indent: Indentation for formatting
        """
        if node is None:
            # Find root node (should be the last one added)
            root_node = list(tree.keys())[-1] if tree else None
            if root_node:
                Phylogenetics.print_tree(tree, root_node, "")
        else:
            if node in tree:
                node_data = tree[node]
                print(f"{indent}{node}:")
                if 'node' in node_data:  # Leaf node
                    print(f"{indent}  {node_data['node']} [{node_data['distance']:.3f}]")
                else:  # Internal node
                    if 'left' in node_data:
                        print(f"{indent}  {node_data['left']['node']} [{node_data['left']['distance']:.3f}]")
                    if 'right' in node_data:
                        print(f"{indent}  {node_data['right']['node']} [{node_data['right']['distance']:.3f}]")


class MolecularEvolution:
    """Molecular evolution analysis and models."""
    
    # DNA substitution rates for Jukes-Cantor model
    JUKES_CANTOR_RATES = {
        ('A', 'T'): 1.0, ('A', 'G'): 1.0, ('A', 'C'): 1.0,
        ('T', 'G'): 1.0, ('T', 'C'): 1.0, ('G', 'C'): 1.0
    }
    
    @staticmethod
    def jukes_cantor_distance(seq1: str, seq2: str) -> float:
        """
        Calculate Jukes-Cantor distance between two DNA sequences.
        
        Args:
            seq1: First DNA sequence
            seq2: Second DNA sequence
            
        Returns:
            Jukes-Cantor distance
        """
        if len(seq1) != len(seq2):
            raise ValueError("Sequences must be same length")
        
        # Count differences
        differences = sum(c1 != c2 for c1, c2 in zip(seq1, seq2))
        p = differences / len(seq1)  # Proportion of differences
        
        if p >= 0.75:
            return float('inf')  # Maximum distance for Jukes-Cantor
        
        # Jukes-Cantor correction
        distance = -0.75 * math.log(1 - 4*p/3)
        return distance
    
    @staticmethod
    def kimura_distance(seq1: str, seq2: str) -> Dict:
        """
        Calculate Kimura 2-parameter distance.
        
        Args:
            seq1: First DNA sequence
            seq2: Second DNA sequence
            
        Returns:
            Dictionary with transition/transversion counts and distance
        """
        if len(seq1) != len(seq2):
            raise ValueError("Sequences must be same length")
        
        transitions = 0
        transversions = 0
        
        for c1, c2 in zip(seq1, seq2):
            if c1 != c2:
                is_transition = (c1 in 'AG' and c2 in 'AG') or (c1 in 'CT' and c2 in 'CT')
                if is_transition:
                    transitions += 1
                else:
                    transversions += 1
        
        n = len(seq1)
        P = transitions / n  # Proportion of transitions
        Q = transversions / n  # Proportion of transversions
        
        # Kimura 2-parameter distance
        if (1 - 2*P - Q) <= 0 or (1 - 2*Q) <= 0:
            distance = float('inf')
        else:
            distance = -0.5 * math.log(1 - 2*P - Q) - 0.25 * math.log(1 - 2*Q)
        
        return {
            'distance': distance,
            'transitions': transitions,
            'transversions': transversions,
            'P': P,
            'Q': Q
        }
    
    @staticmethod
    def generate_sequences_with_substitution(n_sequences: int, sequence_length: int,
                                            substitution_rate: float = 0.01,
                                            ancestral_sequence: Optional[str] = None) -> List[str]:
        """
        Generate sequences with molecular evolution.
        
        Args:
            n_sequences: Number of descendant sequences
            sequence_length: Length of sequences
            substitution_rate: Rate of substitution per site
            ancestral_sequence: Starting sequence (random if None)
            
        Returns:
            List of evolved sequences
        """
        # Generate ancestral sequence
        if ancestral_sequence is None:
            bases = ['A', 'T', 'G', 'C']
            ancestral_sequence = ''.join(random.choice(bases) for _ in range(sequence_length))
        
        sequences = [ancestral_sequence]
        
        # DNA substitution matrix
        substitution_matrix = {
            'A': {'T': substitution_rate/3, 'G': substitution_rate/3, 'C': substitution_rate/3},
            'T': {'A': substitution_rate/3, 'G': substitution_rate/3, 'C': substitution_rate/3},
            'G': {'A': substitution_rate/3, 'T': substitution_rate/3, 'C': substitution_rate/3},
            'C': {'A': substitution_rate/3, 'T': substitution_rate/3, 'G': substitution_rate/3}
        }
        
        # Generate descendant sequences
        for _ in range(n_sequences - 1):
            current_seq = list(sequences[-1])  # Start from last sequence
            
            # Apply substitutions
            for i in range(len(current_seq)):
                if random.random() < substitution_rate:
                    current_base = current_seq[i]
                    possible_substitutions = substitution_matrix[current_base]
                    new_base = random.choices(
                        list(possible_substitutions.keys()),
                        weights=list(possible_substitutions.values())
                    )[0]
                    current_seq[i] = new_base
            
            sequences.append(''.join(current_seq))
        
        return sequences
    
    @staticmethod
    def calculate_substitution_rate_matrix() -> np.ndarray:
        """
        Calculate substitution rate matrix for DNA.
        
        Returns:
            4x4 substitution rate matrix
        """
        # Jukes-Cantor model
        bases = ['A', 'C', 'G', 'T']
        rates = np.zeros((4, 4))
        
        for i, base1 in enumerate(bases):
            for j, base2 in enumerate(bases):
                if i != j:
                    rates[i, j] = 1.0  # Equal substitution rate to all other bases
        
        return rates
    
    @staticmethod
    def simulate_molecular_clock(ancestral_sequence: str, n_lineages: int,
                               time_points: List[float], substitution_rate: float) -> Dict:
        """
        Simulate molecular evolution under molecular clock.
        
        Args:
            ancestral_sequence: Starting sequence
            n_lineages: Number of lineages to simulate
            time_points: Time points for sampling
            substitution_rate: Substitution rate per unit time
            
        Returns:
            Dictionary with evolved sequences and tree
        """
        # Create simple bifurcating tree (simplified)
        sequences = {0: ancestral_sequence}
        
        for t in time_points:
            for lineage in range(n_lineages):
                # Generate sequence by applying substitutions over time
                evolved_seq = list(ancestral_sequence)
                
                # Apply substitutions based on time and rate
                n_substitutions = np.random.poisson(substitution_rate * t * len(evolved_seq))
                
                for _ in range(int(n_substitutions)):
                    pos = random.randint(0, len(evolved_seq) - 1)
                    current_base = evolved_seq[pos]
                    
                    # Random substitution
                    new_bases = [b for b in ['A', 'T', 'G', 'C'] if b != current_base]
                    evolved_seq[pos] = random.choice(new_bases)
                
                sequences[(t, lineage)] = ''.join(evolved_seq)
        
        return {
            'sequences': sequences,
            'time_points': time_points,
            'substitution_rate': substitution_rate
        }


class GenomeAnalysis:
    """Basic genome analysis tools."""
    
    @staticmethod
    def kmer_frequency(sequence: str, k: int) -> Dict[str, int]:
        """
        Calculate k-mer frequencies in a sequence.
        
        Args:
            sequence: DNA sequence
            k: Length of k-mers
            
        Returns:
            Dictionary with k-mer frequencies
        """
        if k > len(sequence):
            return {}
        
        kmers = {}
        for i in range(len(sequence) - k + 1):
            kmer = sequence[i:i+k]
            kmers[kmer] = kmers.get(kmer, 0) + 1
        
        return kmers
    
    @staticmethod
    def find_repeats(sequence: str, min_length: int = 5, max_length: Optional[int] = None) -> List[Dict]:
        """
        Find repeats in a DNA sequence.
        
        Args:
            sequence: DNA sequence
            min_length: Minimum repeat length
            max_length: Maximum repeat length
            
        Returns:
            List of repeat regions
        """
        if max_length is None:
            max_length = len(sequence) // 2
        
        repeats = []
        
        for length in range(min_length, max_length + 1):
            for i in range(len(sequence) - 2*length + 1):
                seq1 = sequence[i:i+length]
                seq2 = sequence[i+length:i+2*length]
                
                if seq1 == seq2:
                    repeats.append({
                        'start': i,
                        'end': i + 2*length,
                        'repeat_unit': seq1,
                        'length': length
                    })
        
        return repeats
    
    @staticmethod
    def calculate_gc_skew(sequence: str, window_size: int = 1000) -> List[float]:
        """
        Calculate GC skew along a sequence.
        
        Args:
            sequence: DNA sequence
            window_size: Size of sliding window
            
        Returns:
            List of GC skew values
        """
        skews = []
        
        for i in range(0, len(sequence), window_size):
            window = sequence[i:i+window_size]
            
            g_count = window.count('G')
            c_count = window.count('C')
            
            if g_count + c_count > 0:
                skew = (g_count - c_count) / (g_count + c_count)
            else:
                skew = 0
            
            skews.append(skew)
        
        return skews
    
    @staticmethod
    def find_oris(sequence: str, gc_skew_threshold: float = 0.1) -> List[int]:
        """
        Find potential origins of replication based on GC skew.
        
        Args:
            sequence: DNA sequence
            gc_skew_threshold: Threshold for GC skew change
            
        Returns:
            List of potential oriC positions
        """
        gc_skews = GenomeAnalysis.calculate_gc_skew(sequence)
        
        # Find positions where GC skew changes sign
        ori_candidates = []
        
        for i in range(1, len(gc_skews)):
            if abs(gc_skews[i] - gc_skews[i-1]) > gc_skew_threshold:
                if gc_skews[i-1] * gc_skews[i] < 0:  # Sign change
                    ori_candidates.append(i)
        
        return ori_candidates


def demo_bioinformatics_tools():
    """Demonstrate bioinformatics tools."""
    print("Computational Biology and Bioinformatics - Educational Examples")
    print("=" * 65)
    
    # Example 1: Sequence Analysis
    print("\n1. DNA Sequence Analysis:")
    dna_seq = "ATGCGTACGTTAGCTAGCTAGCTAGCGATCGATCG"
    
    print(f"Sequence: {dna_seq}")
    print(f"GC Content: {SequenceAnalysis.gc_content(dna_seq):.3f}")
    print(f"Reverse Complement: {SequenceAnalysis.reverse_complement(dna_seq)}")
    
    # Transcription and translation
    rna_seq = SequenceAnalysis.transcribe(dna_seq)
    protein = SequenceAnalysis.translate(rna_seq)
    print(f"RNA Sequence: {rna_seq}")
    print(f"Translated Protein: {protein}")
    
    # ORF finding
    long_seq = "ATGAAATAGATGTAGATGTAGATGAAATAG" * 5
    orfs = SequenceAnalysis.find_orfs(long_seq, min_length=30)
    print(f"Found {len(orfs)} ORFs")
    
    # Example 2: Sequence Alignment
    print("\n2. Sequence Alignment:")
    seq1 = "ACGTACGT"
    seq2 = "ACGTCGT"
    
    print(f"Sequence 1: {seq1}")
    print(f"Sequence 2: {seq2}")
    
    # Global alignment
    global_alignment = SequenceAlignment.needleman_wunsch(seq1, seq2)
    print(f"Global Alignment Score: {global_alignment['score']}")
    print(f"Aligned 1: {global_alignment['aligned_seq1']}")
    print(f"Aligned 2: {global_alignment['aligned_seq2']}")
    
    # Local alignment
    local_alignment = SequenceAlignment.smith_waterman(seq1, seq2)
    print(f"Local Alignment Score: {local_alignment['score']}")
    
    # Example 3: Phylogenetic Analysis
    print("\n3. Phylogenetic Analysis:")
    sequences = [
        "ACGTACGT",
        "ACGTACGA",
        "ACGTACGG",
        "ACGTACTT"
    ]
    labels = ["Species_A", "Species_B", "Species_C", "Species_D"]
    
    # Calculate distance matrix
    distance_matrix = Phylogenetics.calculate_distance_matrix(sequences)
    print("Distance Matrix:")
    print(distance_matrix)
    
    # Build phylogenetic tree
    tree = Phylogenetics.neighbor_joining(distance_matrix, labels)
    print("\nPhylogenetic Tree:")
    Phylogenetics.print_tree(tree)
    
    # Example 4: Molecular Evolution
    print("\n4. Molecular Evolution:")
    
    # Generate sequences with evolution
    ancestral = "ATG" * 10  # 30 bp sequence
    evolved_seqs = MolecularEvolution.generate_sequences_with_substitution(
        5, 30, substitution_rate=0.05, ancestral_sequence=ancestral
    )
    
    print(f"Ancestral sequence: {ancestral}")
    for i, seq in enumerate(evolved_seqs[1:], 1):
        print(f"Evolved {i}: {seq}")
    
    # Calculate evolutionary distances
    jc_distance = MolecularEvolution.jukes_cantor_distance(ancestral, evolved_seqs[1])
    kimura_result = MolecularEvolution.kimura_distance(ancestral, evolved_seqs[1])
    
    print(f"Jukes-Cantor distance: {jc_distance:.3f}")
    print(f"Kimura distance: {kimura_result['distance']:.3f}")
    print(f"Transitions: {kimura_result['transitions']}, Transversions: {kimura_result['transversions']}")
    
    # Example 5: Genome Analysis
    print("\n5. Genome Analysis:")
    genome_seq = "ATGC" * 100 + "GCGC" * 50 + "ATGC" * 100
    
    # K-mer analysis
    kmer_freq = GenomeAnalysis.kmer_frequency(genome_seq, 4)
    print(f"Number of 4-mers: {len(kmer_freq)}")
    print(f"Top 5 4-mers: {dict(list(sorted(kmer_freq.items(), key=lambda x: x[1], reverse=True))[:5])}")
    
    # GC skew analysis
    gc_skews = GenomeAnalysis.calculate_gc_skew(genome_seq, window_size=200)
    print(f"GC skew range: {min(gc_skews):.3f} to {max(gc_skews):.3f}")
    
    # Repeat finding
    repeat_seq = "ATGC" * 20 + "ATGC" * 20
    repeats = GenomeAnalysis.find_repeats(repeat_seq, min_length=10)
    print(f"Found {len(repeats)} repeats")
    
    print("\nNote: This implementation provides educational examples of bioinformatics algorithms.")
    print("For production use, consider specialized tools like Biopython, MAFFT, BLAST, etc.")


if __name__ == "__main__":
    demo_bioinformatics_tools()
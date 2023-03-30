use std::cmp::Ordering;
use std::rc::{Rc, Weak};

struct MemoryBlock {
    pages: Vec<usize>,
    free: bool,
    next_block: Option<Weak<MemoryBlock>>,
    next_block_size: usize,
    refs: usize,
}

impl MemoryBlock {
    fn new(pages: Vec<usize>, free: bool, next_block: Option<Weak<MemoryBlock>>, next_block_size: usize) -> MemoryBlock {
        MemoryBlock {
            pages,
            free,
            next_block,
            next_block_size,
            refs: 0,
        }
    }

    fn size(&self) -> usize {
        self.pages.len() * 4096 // 4096 bytes = 4 kilobytes
    }

    fn add_ref(&mut self) {
        self.refs += 1;
    }

    fn remove_ref(&mut self) {
        self.refs -= 1;
    }
}

#[derive(Debug)]
struct AVLTree<K: Ord, V> {
    root: Option<Box<Node<K, V>>>,
}

#[derive(Debug)]
struct Node<K: Ord, V> {
    key: K,
    value: V,
    memory:  MemoryBlock,
    height: i32,
    left: Option<Box<Node<K, V>>>,
    right: Option<Box<Node<K, V>>>,
}

impl<K: Ord, V> AVLTree<K, V> {
    fn new() -> Self {
        println!("anyone");
        AVLTree { root: None };
        println!("new tree made");
    }

    fn height(node: &Option<Box<Node<K, V>>>) -> i32 {
        node.as_ref().map_or(-1, |n| n.height)
    }

    fn rotate_right(mut node: Box<Node<K, V>>) -> Box<Node<K, V>> {
        let mut new_root = node.left.take().unwrap();
        node.left = new_root.right.take();
        new_root.right = Some(node);

        node.height = 1 + std::cmp::max(Self::height(&node.left), Self::height(&node.right));
        new_root.height =
            1 + std::cmp::max(Self::height(&new_root.left), Self::height(&new_root.right));

        new_root
    }

    fn rotate_left(mut node: Box<Node<K, V>>) -> Box<Node<K, V>> {
        let mut new_root = node.right.take().unwrap();
        node.right = new_root.left.take();
        new_root.left = Some(node);

        node.height = 1 + std::cmp::max(Self::height(&node.left), Self::height(&node.right));
        new_root.height =
            1 + std::cmp::max(Self::height(&new_root.left), Self::height(&new_root.right));

        new_root
    }

    fn balance(mut node: Box<Node<K, V>>) -> Box<Node<K, V>> {
        let lh = Self::height(&node.left);
        let rh = Self::height(&node.right);

        if lh - rh > 1 {
            let left = node.left.take().unwrap();
            if Self::height(&left.left) < Self::height(&left.right) {
                node.left = Some(Self::rotate_left(left));
            }
            return Self::rotate_right(node);
        }

        if rh - lh > 1 {
            let right = node.right.take().unwrap();
            if Self::height(&right.right) < Self::height(&right.left) {
                node.right = Some(Self::rotate_right(right));
            }
            return Self::rotate_left(node);
        }

        node.height = 1 + std::cmp::max(Self::height(&node.left), Self::height(&node.right));
        node
    }

    fn insert_node(
        node: &mut Option<Box<Node<K, V>>>,
        key: K,
        value: V,
    ) -> Option<Box<Node<K, V>>> {
        if let Some(node) = node {
            match key.cmp(&node.key) {
                Ordering::Less => node.left = Self::insert_node(&mut node.left, key, value),
                Ordering::Greater => node.right = Self::insert_node(&mut node.right, key, value),
                Ordering::Equal => {
                    node.value = value;
                    return Some(node.clone());
                }
            }
            Some(Self::balance(node.clone()))
        } else {
            Some(Box::new(Node {
                key,
                value,
                height: 0,
                left: None,
                right: None,
            }));
        }
    }

    fn remove_node(node: &mut Option<Box<Node<K, V>>>, key: &K) -> Option<Box<Node<K, V>>> {
        if let Some(ref mut node) = node {
            match key.cmp(&node.key) {
                Ordering::Less => node.left = Self::remove_node(&mut node.left, key),
                Ordering::Greater => node.right = Self::remove_node(&mut node.right, key),
                Ordering::Equal => {
                    if node.left.is_none() {
                        return node.right.take();
                    } else if node.right.is_none() {
                        return node.left.take();
                    } else {
                        let mut right = node.right.take().unwrap();
                        let mut min = right.as_mut();

                        while min.left.is_some() {
                            min = min.left.as_mut().unwrap();
                        }

                        std::mem::swap(&mut node.key, &mut min.key);
                        std::mem::swap(&mut node.value, &mut min.value);

                        node.right = Self::remove_node(&mut node.right, &min.key);
                    }
                }
            }

            Some(Self::balance(node.clone()))
        } else {
            None
        }
    }

    fn search(&self, key: &K) -> Option<&V> {
        let mut current = &self.root;

        while let Some(node) = current {
            match key.cmp(&node.key) {
                Ordering::Less => current = &node.left,
                Ordering::Greater => current = &node.right,
                Ordering::Equal => return Some(&node.value),
            }
        }

        None
    }

    fn remove(&mut self, key: K) -> Option<V> {
        let removed = Self::remove_node(&mut self.root, &key);
        removed.map(|node| node.value)
    }

    fn insert(&mut self, key: K, value: V) {
        self.root = Self::insert_node(&mut self.root, key, value);
    }
}

#[derive(Debug)]
struct Allocator {
    memory_tree: AVLTree<usize, usize>,
}

impl Allocator {
    pub fn new() -> Self {
        const USER_MEM_START: usize = 0x10000000;
        const USER_MEM_SIZE: usize = 0x10000000;

        const MAX_BLOCK_SIZE_EXP: u32 = 20;
        const MIN_BLOCK_SIZE_EXP: u32 = 12;

        let mut memory_tree = AVLTree::<usize, usize>::new();

        let mut remaining_memory = USER_MEM_SIZE;
        let mut current_address = USER_MEM_START;

        while remaining_memory > 0 {
            let mut block_size_exp = MIN_BLOCK_SIZE_EXP;
            let max_exp = MAX_BLOCK_SIZE_EXP.min((remaining_memory as f64).log2().floor() as u32);

            let exp_range = MIN_BLOCK_SIZE_EXP..=max_exp;
            let exp_weights = exp_range
                .clone()
                .map(|x| (MAX_BLOCK_SIZE_EXP - x) as f64)
                .collect::<Vec<_>>();
            let sum_weights = exp_weights.iter().sum::<f64>();

            let random_weight: f64 = rand::random::<f64>() * sum_weights;

            let mut cumulative_weight = 0.0;
            for (exp, &weight) in exp_range.zip(exp_weights.iter()) {
                cumulative_weight += weight;
                if cumulative_weight >= random_weight {
                    block_size_exp = exp;
                    break;
                }
            }

            let block_size = 1 << block_size_exp;
            let block_end = current_address + block_size - 1;
            memory_tree.insert(block_size, current_address, current_address, block_end);
            current_address += block_size;
            remaining_memory -= block_size;
        }

        Allocator { memory_tree }
    }

    pub fn allocate_block(&self, size: usize) -> Option<(usize, usize)> {
        let mut current = &self.memory_tree.root;

        while let Some(node) = current {
            if node.key >= size {
                return Some((node.start, node.end));
            } else {
                match size.cmp(&node.key) {
                    Ordering::Less => current = &node.left,
                    _ => current = &node.right,
                }
            }
        }

        None
    }
}

// ... (AVLTree implementation) ...

// Update the insert method in AVLTree to accommodate the start and end boundaries
fn insert(&mut self, key: K, value: V, start: usize, end: usize) {
    self.root = Self::insert_node(&mut self.root, key, value, start, end);
}

// Update the insert_node method in AVLTree to accept start and end parameters
fn insert_node(
    node: &mut Option<Box<Node<K, V>>>,
    key: K,
    value: V,
    start: usize,
    end: usize,
) -> Option<Box<Node<K, V>>> {
    if let Some(node) = node {
        match key.cmp(&node.key) {
            Ordering::Less => node.left = Self::insert_node(&mut node.left, key, value, start, end),
            Ordering::Greater => {
                node.right = Self::insert_node(&mut node.right, key, value, start, end)
            }
            Ordering::Equal => {
                node.value = value;
                node.start = start;
                node.end = end;
                return Some(node.clone());
            }
        }
        Some(Self::balance(node.clone()))
    } else {
        Some(Box::new(Node {
            key,
            value,
            start,
            end,
            height: 0,
            left: None,
            right: None,
        }))
    }
}

// Example usage
fn main() {
    let allocator = Allocator::new();
    let requested_size = 4096;
    let allocated_block = allocator.allocate_block(requested_size);
    match allocated_block {
        Some((start, end)) => {
            println!(
                "Allocated block for size {}: start = 0x{:x}, end = 0x{:x}",
                requested_size, start, end
            );
        }
        None => {
            println!("No suitable block found for size {}", requested_size);
        }
    }
}

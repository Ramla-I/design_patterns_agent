# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 9
- **Temporal ordering**: 0
- **Resource lifecycle**: 1
- **State machine**: 2
- **Precondition**: 3
- **Protocol**: 3
- **Modules analyzed**: 1

## Resource Lifecycle Invariants

### 4. Graph reference-counted liveness protocol (ref_count must not underflow; deletion is only valid after prior retain/copy)

**Location**: `/data/test_case/main.rs:1-197`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: Nodes inside Graph have an implicit lifetime governed by a manual `ref_count`. Operations like `shallow_copy` increment counts transitively, and `delete_node` decrements the count. The code relies on the invariant that `delete_node` is only called when `ref_count > 0`; otherwise `ref_count -= 1` can underflow (or violate intended semantics). There is also an implicit 'retain/release' protocol: if you perform a copy/retain (`shallow_copy`), you must later balance it with corresponding deletions. None of this is enforced by types: callers can call `delete_node` arbitrarily, and `ref_count` is a plain integer field on `Node` updated without checked arithmetic.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Node, impl Node (2 methods); struct Input, impl Input (3 methods); 2 free function(s)

    }
}

struct Graph {
    nodes: Vec<Node>,
    // Keyed by C-string bytes (up to first NUL) exactly like strcmp would see.
    name_to_index: HashMap<Vec<u8>, usize>,
}

impl Graph {
    fn new() -> Self {
        Self {
            nodes: Vec::new(),
            name_to_index: HashMap::new(),
        }
    }

    fn make_name_buf_from_input_line(line: &str) -> ([u8; MAX_CITY_NAME], Vec<u8>) {
        // C code copies exactly MAX_CITY_NAME-1 bytes from the provided pointer,
        // then sets last byte to NUL. That means:
        // - If input is longer than 63 bytes, stored name is first 63 bytes.
        // - If input is shorter, it still copies bytes beyond end in C (UB),
        //   but in the interactive program this would be a proper C string.
        // For our executable, we treat the line as the C string content and copy up to 63 bytes.
        let mut buf = [0u8; MAX_CITY_NAME];
        let bytes = line.as_bytes();
        let copy_len = bytes.len().min(MAX_CITY_NAME - 1);
        buf[..copy_len].copy_from_slice(&bytes[..copy_len]);
        buf[MAX_CITY_NAME - 1] = 0;

        // Key is bytes up to first NUL (which will be at copy_len unless input had embedded NUL).
        let nul = buf.iter().position(|&b| b == 0).unwrap_or(MAX_CITY_NAME);
        (buf, buf[..nul].to_vec())
    }

    fn add_node(&mut self, city_line: &str) -> Result<String, String> {
        if city_line.is_empty() {
            return Err("Error: NULL parameter in add_node".to_string());
        }
        if self.nodes.len() >= MAX_NODES {
            return Err(format!("Error: Graph is full (max {MAX_NODES} nodes)"));
        }

        let (name_buf, key) = Self::make_name_buf_from_input_line(city_line);
        if self.name_to_index.contains_key(&key) {
            let shown = String::from_utf8_lossy(&key).to_string();
            return Err(format!("Error: Node '{}' already exists", shown));
        }

        let idx = self.nodes.len();
        self.nodes.push(Node {
            name_buf,
            ref_count: 1,
            edges: Vec::new(),
        });
        self.name_to_index.insert(key.clone(), idx);

        Ok(String::from_utf8_lossy(&key).to_string())
    }

    fn get_node_by_name(&self, city_line: &str) -> Option<usize> {
        if city_line.is_empty() {
            return None;
        }
        let (_buf, key) = Self::make_name_buf_from_input_line(city_line);
        self.name_to_index.get(&key).copied()
    }

    fn add_edge(&mut self, from: usize, to: usize, distance: i32) -> Result<(), String> {
        if from >= self.nodes.len() || to >= self.nodes.len() {
            return Err("Error: NULL node in add_edge".to_string());
        }
        if self.nodes[from].edges.len() >= MAX_EDGES {
            let name = self.nodes[from].name_string_lossy();
            return Err(format!("Error: Node '{}' has maximum edges", name));
        }
        if distance < 0 {
            return Err("Error: Negative distance not allowed".to_string());
        }
        if self.nodes[from].edges.iter().any(|&(dst, _)| dst == to) {
            return Err("Error: Edge already exists".to_string());
        }
        self.nodes[from].edges.push((to, distance));
        Ok(())
    }

    fn shallow_copy(&mut self, start: usize) -> Result<(), String> {
        if start >= self.nodes.len() {
            return Err("Error: NULL node in shallow_copy".to_string());
        }

        // Increment refs recursively with visited set.
        let mut visited = vec![false; self.nodes.len().min(MAX_NODES)];
        let mut q = VecDeque::new();
        q.push_back(start);

        while let Some(u) = q.pop_front() {
            if u >= visited.len() || visited[u] {
                continue;
            }
            visited[u] = true;
            self.nodes[u].ref_count += 1;
            for &(v, _) in &self.nodes[u].edges {
                q.push_back(v);
            }
        }
        Ok(())
    }

    fn delete_node(&mut self, idx: usize) -> Result<(), String> {
        if idx >= self.nodes.len() {
            return Err("Error: NULL node in delete_node".to_string());
        }
        self.nodes[idx].ref_count -= 1;
        Ok(())
    }

    fn find_shortest_path(&self, start: usize, end: usize) -> Result<Vec<usize>, String> {
        if start >= self.nodes.len() || end >= self.nodes.len() {
            return Err("Error: NULL parameter in find_shortest_path".to_string());
        }

        let n = self.nodes.len().min(MAX_NODES);
        let mut dist = vec![INT_MAX; n];
        let mut prev: Vec<Option<usize>> = vec![None; n];
        let mut visited = vec![false; n];

        dist[start] = 0;

        loop {
            let mut current: Option<usize> = None;
            let mut min_d = INT_MAX;
            for i in 0..n {
                if !visited[i] && dist[i] < min_d {
                    min_d = dist[i];
                    current = Some(i);
                }
            }
            let Some(u) = current else { break };
            visited[u] = true;
            if u == end {
                break;
            }

            for &(v, w) in &self.nodes[u].edges {
                if v >= n {
                    continue;
                }
                let nd = dist[u].saturating_add(w);
                if nd < dist[v] {
                    dist[v] = nd;
                    prev[v] = Some(u);
                }
            }
        }

        if end >= n || dist[end] == INT_MAX {
            return Err("No path found".to_string());
        }

        let mut path_rev = Vec::new();
        let mut cur = Some(end);
        while let Some(u) = cur {
            path_rev.push(u);
            cur = prev[u];
        }
        path_rev.reverse();
        Ok(path_rev)
    }

    fn print_node(&self, idx: usize) {
        if idx >= self.nodes.len() {
            println!("NULL node");
            return;
        }
        let node = &self.nodes[idx];
        println!("City: {} (ref_count: {})", node.name_string_lossy(), node.ref_count);
        println!("  Edges:");
        for &(dst, dist) in &node.edges {
            let dst_name = self
                .nodes
                .get(dst)
                .map(|n| n.name_string_lossy())
                .unwrap_or_else(|| "NULL".to_string());
            println!("    -> {} (distance: {})", dst_name, dist);
        }
    }

    fn print_graph(&self) {
        println!("Graph with {} nodes:", self.nodes.len());
        for i in 0..self.nodes.len() {
            self.print_node(i);
        }
    }
}

```

**Entity:** Graph

**States:** Live(ref_count>0), Dead(ref_count==0)

**Transitions:**
- Live(ref_count>0) -> Live(ref_count+1) via `shallow_copy(start)` (increments visited nodes' `ref_count`)
- Live(ref_count>0) -> Live(ref_count-1) via `delete_node(idx)`
- Live(ref_count==1) -> Dead(ref_count==0) via `delete_node(idx)`

**Evidence:** method `add_node`: initializes `ref_count: 1` in pushed `Node { ..., ref_count: 1, ... }`; method `shallow_copy`: `self.nodes[u].ref_count += 1;` indicates a retain-like operation over a reachable subgraph; method `delete_node`: `self.nodes[idx].ref_count -= 1;` with no guard/check for zero, implying a precondition `ref_count > 0`; method `print_node`: prints `(ref_count: {})`, suggesting this field is semantically meaningful state

**Implementation:** Model refcount changes with an explicit token representing a retained reference, e.g. `struct NodeRef { id: NodeId }` that is only constructible by `Graph::add_node` / `Graph::shallow_copy` and whose `Drop` releases (decrements) automatically (RAII), or an explicit `release(self)` consuming method. This makes it impossible to call `delete_node` without owning a corresponding capability, and can prevent underflow by construction.

---

## State Machine Invariants

### 8. Node lifecycle via manual ref_count (Alive / LogicallyDeleted) and non-negative refcount invariant

**Location**: `/data/test_case/main.rs:1-445`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: Nodes participate in a manual lifetime/ownership scheme encoded as `ref_count: i32`. `shallow_copy` increments `ref_count` for a reachable subgraph, and `delete_node` decrements it, implying an intended invariant that ref_count should not go negative and that nodes with ref_count==0 are 'deleted' (or at least no longer referenced). However, the type system does not prevent calling `delete_node` multiple times (driving counts negative), does not prevent using nodes with ref_count<=0 in other operations, and does not represent the 'deleted' state distinctly.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Node, impl Node (2 methods); struct Graph, impl Graph (10 methods); struct Input, impl Input (3 methods)

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![feature(c_variadic)]
#![feature(extern_types)]
#![feature(linkage)]
#![feature(rustc_private)]
#![feature(thread_local)]
#![feature(formatting_options)]

use std::collections::{HashMap, VecDeque};
use std::io::{self, Read};

const MAX_CITY_NAME: usize = 64; // C buffer size, includes NUL
const MAX_EDGES: usize = 10;
const MAX_NODES: usize = 100;
const INT_MAX: i32 = i32::MAX;

#[derive(Clone)]
struct Node {
    // Store as raw bytes (C-like), may include any bytes except we keep it as entered (minus newline),
    // and comparisons/printing use C-string semantics (stop at first NUL).
    name_buf: [u8; MAX_CITY_NAME], // always NUL-terminated somewhere
    ref_count: i32,
    edges: Vec<(usize, i32)>, // (destination index, distance)
}

impl Node {
    fn name_cstr_bytes(&self) -> &[u8] {
        let nul = self.name_buf.iter().position(|&b| b == 0).unwrap_or(MAX_CITY_NAME);
        &self.name_buf[..nul]
    }
    fn name_string_lossy(&self) -> String {
        String::from_utf8_lossy(self.name_cstr_bytes()).to_string()
    }
}

struct Graph {
    nodes: Vec<Node>,
    // Keyed by C-string bytes (up to first NUL) exactly like strcmp would see.
    name_to_index: HashMap<Vec<u8>, usize>,
}

impl Graph {
    fn new() -> Self {
        Self {
            nodes: Vec::new(),
            name_to_index: HashMap::new(),
        }
    }

    fn make_name_buf_from_input_line(line: &str) -> ([u8; MAX_CITY_NAME], Vec<u8>) {
        // C code copies exactly MAX_CITY_NAME-1 bytes from the provided pointer,
        // then sets last byte to NUL. That means:
        // - If input is longer than 63 bytes, stored name is first 63 bytes.
        // - If input is shorter, it still copies bytes beyond end in C (UB),
        //   but in the interactive program this would be a proper C string.
        // For our executable, we treat the line as the C string content and copy up to 63 bytes.
        let mut buf = [0u8; MAX_CITY_NAME];
        let bytes = line.as_bytes();
        let copy_len = bytes.len().min(MAX_CITY_NAME - 1);
        buf[..copy_len].copy_from_slice(&bytes[..copy_len]);
        buf[MAX_CITY_NAME - 1] = 0;

        // Key is bytes up to first NUL (which will be at copy_len unless input had embedded NUL).
        let nul = buf.iter().position(|&b| b == 0).unwrap_or(MAX_CITY_NAME);
        (buf, buf[..nul].to_vec())
    }

    fn add_node(&mut self, city_line: &str) -> Result<String, String> {
        if city_line.is_empty() {
            return Err("Error: NULL parameter in add_node".to_string());
        }
        if self.nodes.len() >= MAX_NODES {
            return Err(format!("Error: Graph is full (max {MAX_NODES} nodes)"));
        }

        let (name_buf, key) = Self::make_name_buf_from_input_line(city_line);
        if self.name_to_index.contains_key(&key) {
            let shown = String::from_utf8_lossy(&key).to_string();
            return Err(format!("Error: Node '{}' already exists", shown));
        }

        let idx = self.nodes.len();
        self.nodes.push(Node {
            name_buf,
            ref_count: 1,
            edges: Vec::new(),
        });
        self.name_to_index.insert(key.clone(), idx);

        Ok(String::from_utf8_lossy(&key).to_string())
    }

    fn get_node_by_name(&self, city_line: &str) -> Option<usize> {
        if city_line.is_empty() {
            return None;
        }
        let (_buf, key) = Self::make_name_buf_from_input_line(city_line);
        self.name_to_index.get(&key).copied()
    }

    fn add_edge(&mut self, from: usize, to: usize, distance: i32) -> Result<(), String> {
        if from >= self.nodes.len() || to >= self.nodes.len() {
            return Err("Error: NULL node in add_edge".to_string());
        }
        if self.nodes[from].edges.len() >= MAX_EDGES {
            let name = self.nodes[from].name_string_lossy();
            return Err(format!("Error: Node '{}' has maximum edges", name));
        }
        if distance < 0 {
            return Err("Error: Negative distance not allowed".to_string());
        }
        if self.nodes[from].edges.iter().any(|&(dst, _)| dst == to) {
            return Err("Error: Edge already exists".to_string());
        }
        self.nodes[from].edges.push((to, distance));
        Ok(())
    }

    fn shallow_copy(&mut self, start: usize) -> Result<(), String> {
        if start >= self.nodes.len() {
            return Err("Error: NULL node in shallow_copy".to_string());
        }

        // Increment refs recursively with visited set.
        let mut visited = vec![false; self.nodes.len().min(MAX_NODES)];
        let mut q = VecDeque::new();
        q.push_back(start);

        while let Some(u) = q.pop_front() {
            if u >= visited.len() || visited[u] {
                continue;
            }
            visited[u] = true;
            self.nodes[u].ref_count += 1;
            for &(v, _) in &self.nodes[u].edges {
                q.push_back(v);
            }
        }
        Ok(())
    }

    fn delete_node(&mut self, idx: usize) -> Result<(), String> {
        if idx >= self.nodes.len() {
            return Err("Error: NULL node in delete_node".to_string());
        }
        self.nodes[idx].ref_count -= 1;
        Ok(())
    }

    fn find_shortest_path(&self, start: usize, end: usize) -> Result<Vec<usize>, String> {
        if start >= self.nodes.len() || end >= self.nodes.len() {
            return Err("Error: NULL parameter in find_shortest_path".to_string());
        }

        let n = self.nodes.len().min(MAX_NODES);
        let mut dist = vec![INT_MAX; n];
        let mut prev: Vec<Option<usize>> = vec![None; n];
        let mut visited = vec![false; n];

        dist[start] = 0;

        loop {
            let mut current: Option<usize> = None;
            let mut min_d = INT_MAX;
            for i in 0..n {
                if !visited[i] && dist[i] < min_d {
                    min_d = dist[i];
                    current = Some(i);
                }
            }
            let Some(u) = current else { break };
            visited[u] = true;
            if u == end {
                break;
            }

            for &(v, w) in &self.nodes[u].edges {
                if v >= n {
                    continue;
                }
                let nd = dist[u].saturating_add(w);
                if nd < dist[v] {
                    dist[v] = nd;
                    prev[v] = Some(u);
                }
            }
        }

        if end >= n || dist[end] == INT_MAX {
            return Err("No path found".to_string());
        }

        let mut path_rev = Vec::new();
        let mut cur = Some(end);
        while let Some(u) = cur {
            path_rev.push(u);
            cur = prev[u];
        }
        path_rev.reverse();
        Ok(path_rev)
    }

    fn print_node(&self, idx: usize) {
        if idx >= self.nodes.len() {
            println!("NULL node");
            return;
        }
        let node = &self.nodes[idx];
        println!("City: {} (ref_count: {})", node.name_string_lossy(), node.ref_count);
        println!("  Edges:");
        for &(dst, dist) in &node.edges {
            let dst_name = self
                .nodes
                .get(dst)
                .map(|n| n.name_string_lossy())
                .unwrap_or_else(|| "NULL".to_string());
            println!("    -> {} (distance: {})", dst_name, dist);
        }
    }

    fn print_graph(&self) {
        println!("Graph with {} nodes:", self.nodes.len());
        for i in 0..self.nodes.len() {
            self.print_node(i);
        }
    }
}

struct Input {
    bytes: Vec<u8>,
    pos: usize,
}

impl Input {
    fn new_from_stdin() -> Self {
        let mut buf = Vec::new();
        io::stdin().read_to_end(&mut buf).unwrap();
        Self { bytes: buf, pos: 0 }
    }

    fn read_line_raw(&mut self) -> Option<String> {
        if self.pos >= self.bytes.len() {
            return None;
        }
        let start = self.pos;
        while self.pos < self.bytes.len() {
            let b = self.bytes[self.pos];
            if b == b'\n' || b == b'\r' {
                break;
            }
            self.pos += 1;
        }
        let end = self.pos;
        while self.pos < self.bytes.len() {
            let b = self.bytes[self.pos];
            if b == b'\n' || b == b'\r' {
                self.pos += 1;
            } else {
                break;
            }
        }
        Some(String::from_utf8_lossy(&self.bytes[start..end]).to_string())
    }

    fn read_nonempty_line(&mut self) -> Option<String> {
        while let Some(line) = self.read_line_raw() {
            if !line.is_empty() {
                return Some(line);
            }
        }
        None
    }
}

fn menu_print() {
    print!(
        "\n=== DAG City Route Manager ===\n\
1. Add city (node)\n\
2. Add route (edge)\n\
3. Show all cities\n\
4. Show city details\n\
5. Find shortest path\n\
6. Make shallow copy of subsection\n\
7. Delete node\n\
8. Exit\n\
Choice: "
    );
}

fn main() {
    print!("City Route Management System\nCommands are read from stdin\n");

    let mut input = Input::new_from_stdin();
    let mut graph = Graph::new();

    loop {
        menu_print();

        let Some(choice_line) = input.read_nonempty_line() else {
            print!("Freeing graph and exiting...\n");
            break;
        };

        let choice: i32 = match choice_line.trim().parse() {
            Ok(v) => v,
            Err(_) => {
                print!("Invalid input\n");
                continue;
            }
        };

        match choice {
            1 => {
                print!("Enter city name: ");
                let name_line = input.read_line_raw().unwrap_or_default();
                match graph.add_node(&name_line) {
                    Ok(stored) => print!("Added city: {}\n", stored),
                    Err(msg) => eprint!("{msg}\n"),
                }
            }
            2 => {
                print!("Enter from city: ");
                let from_line = input.read_line_raw().unwrap_or_default();
                print!("Enter to city: ");
                let to_line = input.read_line_raw().unwrap_or_default();
                print!("Enter distance: ");
                let dist_line = input.read_line_raw().unwrap_or_default();

                let distance: i32 = match dist_line.trim().parse() {
                    Ok(v) => v,
                    Err(_) => {
                        print!("Invalid input\n");
                        continue;
                    }
                };

                let from = graph.get_node_by_name(&from_line);
                let to = graph.get_node_by_name(&to_line);
                match (from, to) {
                    (Some(f), Some(t)) => match graph.add_edge(f, t, distance) {
                        Ok(()) => {
                            let (_fb, fk) = Graph::make_name_buf_from_input_line(&from_line);
                            let (_tb, tk) = Graph::make_name_buf_from_input_line(&to_line);
                            let fs = String::from_utf8_lossy(&fk).to_string();
                            let ts = String::from_utf8_lossy(&tk).to_string();
                            print!("Added route: {} -> {} (distance: {})\n", fs, ts, distance);
                        }
                        Err(msg) => eprint!("{msg}\n"),
                    },
                    _ => eprint!("Error: One or both cities not found\n"),
                }
            }
            3 => {
                graph.print_graph();
            }
            4 => {
                print!("Enter city name: ");
                let name_line = input.read_line_raw().unwrap_or_default();
                if let Some(idx) = graph.get_node_by_name(&name_line) {
                    graph.print_node(idx);
                } else {
                    eprint!("Error: City not found\n");
                }
            }
            5 => {
                print!("Enter start city: ");
                let start_line = input.read_line_raw().unwrap_or_default();
                print!("Enter end city: ");
                let end_line = input.read_line_raw().unwrap_or_default();

                let start = graph.get_node_by_name(&start_line);
                let end = graph.get_node_by_name(&end_line);
                match (start, end) {
                    (Some(s), Some(e)) => match graph.find_shortest_path(s, e) {
                        Ok(path) => {
                            let mut total = 0i32;
                            for w in path.windows(2) {
                                let u = w[0];
                                let v = w[1];
                                if let Some((_, d)) =
                                    graph.nodes[u].edges.iter().find(|&&(dst, _)| dst == v)
                                {
                                    total = total.saturating_add(*d);
                                }
                            }
                            print!("Shortest path: ");
                            for (i, &idx) in path.iter().enumerate() {
                                if i != 0 {
                                    print!(" -> ");
                                }
                                print!("{}", graph.nodes[idx].name_string_lossy());
                            }
                            print!("\nTotal distance: {}\n", total);
                        }
                        Err(msg) => eprint!("{msg}\n"),
                    },
                    _ => eprint!("Error: One or both cities not found\n"),
                }
            }
            6 => {
                print!("Enter start city for shallow copy: ");
                let name_line = input.read_line_raw().unwrap_or_default();
                if let Some(idx) = graph.get_node_by_name(&name_line) {
                    match graph.shallow_copy(idx) {
                        Ok(()) => {
                            let (_b, k) = Graph::make_name_buf_from_input_line(&name_line);
                            let shown = String::from_utf8_lossy(&k).to_string();
                            print!("Shallow copy created from: {}\n", shown);
                        }
                        Err(msg) => eprint!("{msg}\n"),
                    }
                } else {
                    eprint!("Error: City not found\n");
                }
            }
            7 => {
                print!("Enter city name to delete: ");
                let name_line = input.read_line_raw().unwrap_or_default();
                if let Some(idx) = graph.get_node_by_name(&name_line) {
                    match graph.delete_node(idx) {
                        Ok(()) => {
                            let (_b, k) = Graph::make_name_buf_from_input_line(&name_line);
                            let shown = String::from_utf8_lossy(&k).to_string();
                            print!("Deleted node (decremented ref_count): {}\n", shown);
                        }
                        Err(msg) => eprint!("{msg}\n"),
                    }
                } else {
                    eprint!("Error: City not found\n");
                }
            }
            8 => {
                print!("Freeing graph and exiting...\n");
                break;
            }
            _ => {
                print!("Invalid input\n");
            }
        }
    }
}
```

**Entity:** Node

**States:** Alive (ref_count > 0), LogicallyDeleted (ref_count <= 0)

**Transitions:**
- Alive -> Alive via Graph::shallow_copy() (increments reachable nodes' ref_count)
- Alive -> LogicallyDeleted via Graph::delete_node() (decrements ref_count; can cross 0)

**Evidence:** Node field: `ref_count: i32` stores lifecycle state at runtime; Graph::shallow_copy(): `self.nodes[u].ref_count += 1;` (recursive increment protocol); Graph::delete_node(): `self.nodes[idx].ref_count -= 1;` with only bounds check; no guard against underflow/<=0; Graph::print_node(): prints `ref_count` but does not treat 0/negative as invalid, showing deleted-ness is not modeled structurally

**Implementation:** Model references as explicit tokens (e.g., `NodeRef` capability) that are cloned/held by clients; `shallow_copy` returns additional `NodeRef`s for the reachable set; dropping a `NodeRef` decrements automatically (RAII). If you want a compile-time state split, make operations that require liveness take `&LiveNode` obtained by validating `ref_count > 0` once, rather than checking (or not checking) ad hoc.

---

### 6. Input cursor protocol (Readable -> EOF) with advancing position

**Location**: `/data/test_case/main.rs:1-50`

**Confidence**: medium

**Suggested Pattern**: typestate

**Description**: Input maintains an internal cursor (pos) into an in-memory byte buffer (bytes). Calls to read_line_raw()/read_nonempty_line() are only meaningful while pos < bytes.len(); once pos reaches bytes.len(), the reader is at EOF and further reads return None. Each successful read advances pos (including consuming trailing newline/carriage-return bytes), so the result of a call depends on all previous calls (temporal dependence). None of this is represented in the type system; all states are encoded by the runtime relationship between pos and bytes.len(), and correctness relies on careful mutation of pos.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Node, impl Node (2 methods); struct Graph, impl Graph (10 methods); 2 free function(s)

    }
}

struct Input {
    bytes: Vec<u8>,
    pos: usize,
}

impl Input {
    fn new_from_stdin() -> Self {
        let mut buf = Vec::new();
        io::stdin().read_to_end(&mut buf).unwrap();
        Self { bytes: buf, pos: 0 }
    }

    fn read_line_raw(&mut self) -> Option<String> {
        if self.pos >= self.bytes.len() {
            return None;
        }
        let start = self.pos;
        while self.pos < self.bytes.len() {
            let b = self.bytes[self.pos];
            if b == b'\n' || b == b'\r' {
                break;
            }
            self.pos += 1;
        }
        let end = self.pos;
        while self.pos < self.bytes.len() {
            let b = self.bytes[self.pos];
            if b == b'\n' || b == b'\r' {
                self.pos += 1;
            } else {
                break;
            }
        }
        Some(String::from_utf8_lossy(&self.bytes[start..end]).to_string())
    }

    fn read_nonempty_line(&mut self) -> Option<String> {
        while let Some(line) = self.read_line_raw() {
            if !line.is_empty() {
                return Some(line);
            }
        }
        None
    }
}

```

**Entity:** Input

**States:** Readable(pos < bytes.len()), EOF(pos >= bytes.len())

**Transitions:**
- Readable -> Readable via read_line_raw() when a line is returned but pos does not reach bytes.len() after advancing
- Readable -> EOF via read_line_raw() when pos advances to bytes.len() (or starts at EOF)
- Readable -> EOF via read_nonempty_line() when it repeatedly calls read_line_raw() until None

**Evidence:** field bytes: Vec<u8> stores the backing buffer; field pos: usize is the cursor/state variable; read_line_raw(): `if self.pos >= self.bytes.len() { return None; }` defines the EOF state boundary; read_line_raw(): loops mutate `self.pos += 1`, consuming content and then consuming trailing '\n'/'\r' bytes; read_nonempty_line(): `while let Some(line) = self.read_line_raw()` demonstrates protocol layering and reliance on read_line_raw() advancing pos

**Implementation:** Model EOF at the type level by returning a value that represents the remaining input: e.g., `fn read_line_raw(self) -> Result<(String, Input), Eof>` (or `Option<(String, Input)>`) so the 'Readable' state is carried by owning `Input` and EOF is a distinct type/branch; alternatively split into `Input<Readable>` and `Input<Eof>` with `read_line_raw(self) -> Result<(String, Input<Readable>), Input<Eof>>`.

---

## Precondition Invariants

### 1. Node name buffer validity + C-string semantics (raw bytes / NUL-terminated / truncated-at-NUL)

**Location**: `/data/test_case/main.rs:1-22`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: Node encodes its name as a fixed-size raw byte buffer with an implicit invariant: the buffer should contain a NUL terminator somewhere ("always NUL-terminated somewhere"), and all comparisons/printing must use C-string semantics (stop at first NUL). This protocol is not enforced by the type system: code can construct/modify Node such that name_buf has no NUL within MAX_CITY_NAME, or contains interior NULs that truncate the logical name unexpectedly. The current accessors partially defend by truncating at the first NUL (or, if none exists, using the full buffer), but that silently violates the stated invariant and can lead to surprising names when converted/compared/printed.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Graph, impl Graph (10 methods); struct Input, impl Input (3 methods); 2 free function(s)

const INT_MAX: i32 = i32::MAX;

#[derive(Clone)]
struct Node {
    // Store as raw bytes (C-like), may include any bytes except we keep it as entered (minus newline),
    // and comparisons/printing use C-string semantics (stop at first NUL).
    name_buf: [u8; MAX_CITY_NAME], // always NUL-terminated somewhere
    ref_count: i32,
    edges: Vec<(usize, i32)>, // (destination index, distance)
}

impl Node {
    fn name_cstr_bytes(&self) -> &[u8] {
        let nul = self.name_buf.iter().position(|&b| b == 0).unwrap_or(MAX_CITY_NAME);
        &self.name_buf[..nul]
    }
    fn name_string_lossy(&self) -> String {
        String::from_utf8_lossy(self.name_cstr_bytes()).to_string()
    }
}

```

**Entity:** Node

**States:** ValidCStringName, InvalidNameBuffer

**Transitions:**
- InvalidNameBuffer -> ValidCStringName via ensuring a NUL byte exists within name_buf (not represented in types/methods here)

**Evidence:** field: name_buf: [u8; MAX_CITY_NAME]; comment on name_buf: "always NUL-terminated somewhere" and "comparisons/printing use C-string semantics (stop at first NUL)"; method: name_cstr_bytes() searches for NUL via position(|&b| b == 0) and uses unwrap_or(MAX_CITY_NAME), implying the invariant may be violated at runtime; method: name_string_lossy() relies on name_cstr_bytes() and thus inherits the implicit C-string truncation semantics

**Implementation:** Replace name_buf with a newtype that can only be constructed from validated data, e.g. struct CityName(CString) or a custom struct CityName([u8; MAX_CITY_NAME]) that enforces "contains at least one NUL" (and possibly "no interior NUL" if desired) in its constructor; expose only safe getters returning &[u8] up to NUL.

---

### 5. C-string name normalization & uniqueness invariant (truncation at MAX_CITY_NAME-1 and NUL-termination define identity)

**Location**: `/data/test_case/main.rs:1-197`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: Graph node identity is not the full input string; it is a normalized C-string-like name: bytes are truncated to `MAX_CITY_NAME-1`, forced NUL-terminated, and the lookup key is bytes up to the first NUL. This means multiple distinct Rust `&str` inputs can map to the same node key (e.g., differing only after 63 bytes, or containing embedded NUL). Correct use of the API relies on always applying the same normalization when adding and looking up nodes; this is currently done by convention via `make_name_buf_from_input_line`, not by types. The type system does not prevent callers from assuming full-string identity or from mixing raw strings with normalized keys.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Node, impl Node (2 methods); struct Input, impl Input (3 methods); 2 free function(s)

    }
}

struct Graph {
    nodes: Vec<Node>,
    // Keyed by C-string bytes (up to first NUL) exactly like strcmp would see.
    name_to_index: HashMap<Vec<u8>, usize>,
}

impl Graph {
    fn new() -> Self {
        Self {
            nodes: Vec::new(),
            name_to_index: HashMap::new(),
        }
    }

    fn make_name_buf_from_input_line(line: &str) -> ([u8; MAX_CITY_NAME], Vec<u8>) {
        // C code copies exactly MAX_CITY_NAME-1 bytes from the provided pointer,
        // then sets last byte to NUL. That means:
        // - If input is longer than 63 bytes, stored name is first 63 bytes.
        // - If input is shorter, it still copies bytes beyond end in C (UB),
        //   but in the interactive program this would be a proper C string.
        // For our executable, we treat the line as the C string content and copy up to 63 bytes.
        let mut buf = [0u8; MAX_CITY_NAME];
        let bytes = line.as_bytes();
        let copy_len = bytes.len().min(MAX_CITY_NAME - 1);
        buf[..copy_len].copy_from_slice(&bytes[..copy_len]);
        buf[MAX_CITY_NAME - 1] = 0;

        // Key is bytes up to first NUL (which will be at copy_len unless input had embedded NUL).
        let nul = buf.iter().position(|&b| b == 0).unwrap_or(MAX_CITY_NAME);
        (buf, buf[..nul].to_vec())
    }

    fn add_node(&mut self, city_line: &str) -> Result<String, String> {
        if city_line.is_empty() {
            return Err("Error: NULL parameter in add_node".to_string());
        }
        if self.nodes.len() >= MAX_NODES {
            return Err(format!("Error: Graph is full (max {MAX_NODES} nodes)"));
        }

        let (name_buf, key) = Self::make_name_buf_from_input_line(city_line);
        if self.name_to_index.contains_key(&key) {
            let shown = String::from_utf8_lossy(&key).to_string();
            return Err(format!("Error: Node '{}' already exists", shown));
        }

        let idx = self.nodes.len();
        self.nodes.push(Node {
            name_buf,
            ref_count: 1,
            edges: Vec::new(),
        });
        self.name_to_index.insert(key.clone(), idx);

        Ok(String::from_utf8_lossy(&key).to_string())
    }

    fn get_node_by_name(&self, city_line: &str) -> Option<usize> {
        if city_line.is_empty() {
            return None;
        }
        let (_buf, key) = Self::make_name_buf_from_input_line(city_line);
        self.name_to_index.get(&key).copied()
    }

    fn add_edge(&mut self, from: usize, to: usize, distance: i32) -> Result<(), String> {
        if from >= self.nodes.len() || to >= self.nodes.len() {
            return Err("Error: NULL node in add_edge".to_string());
        }
        if self.nodes[from].edges.len() >= MAX_EDGES {
            let name = self.nodes[from].name_string_lossy();
            return Err(format!("Error: Node '{}' has maximum edges", name));
        }
        if distance < 0 {
            return Err("Error: Negative distance not allowed".to_string());
        }
        if self.nodes[from].edges.iter().any(|&(dst, _)| dst == to) {
            return Err("Error: Edge already exists".to_string());
        }
        self.nodes[from].edges.push((to, distance));
        Ok(())
    }

    fn shallow_copy(&mut self, start: usize) -> Result<(), String> {
        if start >= self.nodes.len() {
            return Err("Error: NULL node in shallow_copy".to_string());
        }

        // Increment refs recursively with visited set.
        let mut visited = vec![false; self.nodes.len().min(MAX_NODES)];
        let mut q = VecDeque::new();
        q.push_back(start);

        while let Some(u) = q.pop_front() {
            if u >= visited.len() || visited[u] {
                continue;
            }
            visited[u] = true;
            self.nodes[u].ref_count += 1;
            for &(v, _) in &self.nodes[u].edges {
                q.push_back(v);
            }
        }
        Ok(())
    }

    fn delete_node(&mut self, idx: usize) -> Result<(), String> {
        if idx >= self.nodes.len() {
            return Err("Error: NULL node in delete_node".to_string());
        }
        self.nodes[idx].ref_count -= 1;
        Ok(())
    }

    fn find_shortest_path(&self, start: usize, end: usize) -> Result<Vec<usize>, String> {
        if start >= self.nodes.len() || end >= self.nodes.len() {
            return Err("Error: NULL parameter in find_shortest_path".to_string());
        }

        let n = self.nodes.len().min(MAX_NODES);
        let mut dist = vec![INT_MAX; n];
        let mut prev: Vec<Option<usize>> = vec![None; n];
        let mut visited = vec![false; n];

        dist[start] = 0;

        loop {
            let mut current: Option<usize> = None;
            let mut min_d = INT_MAX;
            for i in 0..n {
                if !visited[i] && dist[i] < min_d {
                    min_d = dist[i];
                    current = Some(i);
                }
            }
            let Some(u) = current else { break };
            visited[u] = true;
            if u == end {
                break;
            }

            for &(v, w) in &self.nodes[u].edges {
                if v >= n {
                    continue;
                }
                let nd = dist[u].saturating_add(w);
                if nd < dist[v] {
                    dist[v] = nd;
                    prev[v] = Some(u);
                }
            }
        }

        if end >= n || dist[end] == INT_MAX {
            return Err("No path found".to_string());
        }

        let mut path_rev = Vec::new();
        let mut cur = Some(end);
        while let Some(u) = cur {
            path_rev.push(u);
            cur = prev[u];
        }
        path_rev.reverse();
        Ok(path_rev)
    }

    fn print_node(&self, idx: usize) {
        if idx >= self.nodes.len() {
            println!("NULL node");
            return;
        }
        let node = &self.nodes[idx];
        println!("City: {} (ref_count: {})", node.name_string_lossy(), node.ref_count);
        println!("  Edges:");
        for &(dst, dist) in &node.edges {
            let dst_name = self
                .nodes
                .get(dst)
                .map(|n| n.name_string_lossy())
                .unwrap_or_else(|| "NULL".to_string());
            println!("    -> {} (distance: {})", dst_name, dist);
        }
    }

    fn print_graph(&self) {
        println!("Graph with {} nodes:", self.nodes.len());
        for i in 0..self.nodes.len() {
            self.print_node(i);
        }
    }
}

```

**Entity:** Graph

**States:** RawInputName, NormalizedCNameKey

**Transitions:**
- RawInputName -> NormalizedCNameKey via `make_name_buf_from_input_line(line)`
- NormalizedCNameKey -> (existing node) via `name_to_index.get(&key)` in `get_node_by_name`
- NormalizedCNameKey -> (new unique node) via uniqueness check in `add_node`

**Evidence:** comment on `name_to_index`: `Keyed by C-string bytes (up to first NUL) exactly like strcmp would see.`; method `make_name_buf_from_input_line`: `let copy_len = bytes.len().min(MAX_CITY_NAME - 1);` and `buf[MAX_CITY_NAME - 1] = 0;` (forced truncation + NUL termination); method `make_name_buf_from_input_line`: `let nul = buf.iter().position(|&b| b == 0)...; (buf, buf[..nul].to_vec())` (key is up to first NUL); method `add_node`: rejects duplicates by key: `if self.name_to_index.contains_key(&key) { ... "Node '{}' already exists" ... }`; method `get_node_by_name`: uses same normalization for lookup: `let (_buf, key) = Self::make_name_buf_from_input_line(city_line); self.name_to_index.get(&key)`

**Implementation:** Introduce a `CityName` newtype that stores the normalized representation (e.g., `struct CityName(Vec<u8>)` or `[u8; MAX_CITY_NAME]` plus computed length). Provide `impl TryFrom<&str> for CityName` performing the exact C-string normalization. Change `add_node(&mut self, name: CityName)` and `get_node_by_name(&self, name: &CityName)` (or accept `impl Into<CityName>`), making the identity rules explicit and preventing accidental reliance on raw `&str` equality.

---

### 9. Edge validity invariants (bounded degree, non-negative weight, unique edge) encoded via runtime checks

**Location**: `/data/test_case/main.rs:1-445`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: Graph enforces several edge-set invariants only at runtime: each node may have at most MAX_EDGES outgoing edges; distances must be non-negative; and there must not be duplicate edges `(from,to)`. These are enforced by checks inside `add_edge`, but nothing in the types prevents constructing invalid edges or attempting invalid additions, and callers only learn via `Result`/error strings.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Node, impl Node (2 methods); struct Graph, impl Graph (10 methods); struct Input, impl Input (3 methods)

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![feature(c_variadic)]
#![feature(extern_types)]
#![feature(linkage)]
#![feature(rustc_private)]
#![feature(thread_local)]
#![feature(formatting_options)]

use std::collections::{HashMap, VecDeque};
use std::io::{self, Read};

const MAX_CITY_NAME: usize = 64; // C buffer size, includes NUL
const MAX_EDGES: usize = 10;
const MAX_NODES: usize = 100;
const INT_MAX: i32 = i32::MAX;

#[derive(Clone)]
struct Node {
    // Store as raw bytes (C-like), may include any bytes except we keep it as entered (minus newline),
    // and comparisons/printing use C-string semantics (stop at first NUL).
    name_buf: [u8; MAX_CITY_NAME], // always NUL-terminated somewhere
    ref_count: i32,
    edges: Vec<(usize, i32)>, // (destination index, distance)
}

impl Node {
    fn name_cstr_bytes(&self) -> &[u8] {
        let nul = self.name_buf.iter().position(|&b| b == 0).unwrap_or(MAX_CITY_NAME);
        &self.name_buf[..nul]
    }
    fn name_string_lossy(&self) -> String {
        String::from_utf8_lossy(self.name_cstr_bytes()).to_string()
    }
}

struct Graph {
    nodes: Vec<Node>,
    // Keyed by C-string bytes (up to first NUL) exactly like strcmp would see.
    name_to_index: HashMap<Vec<u8>, usize>,
}

impl Graph {
    fn new() -> Self {
        Self {
            nodes: Vec::new(),
            name_to_index: HashMap::new(),
        }
    }

    fn make_name_buf_from_input_line(line: &str) -> ([u8; MAX_CITY_NAME], Vec<u8>) {
        // C code copies exactly MAX_CITY_NAME-1 bytes from the provided pointer,
        // then sets last byte to NUL. That means:
        // - If input is longer than 63 bytes, stored name is first 63 bytes.
        // - If input is shorter, it still copies bytes beyond end in C (UB),
        //   but in the interactive program this would be a proper C string.
        // For our executable, we treat the line as the C string content and copy up to 63 bytes.
        let mut buf = [0u8; MAX_CITY_NAME];
        let bytes = line.as_bytes();
        let copy_len = bytes.len().min(MAX_CITY_NAME - 1);
        buf[..copy_len].copy_from_slice(&bytes[..copy_len]);
        buf[MAX_CITY_NAME - 1] = 0;

        // Key is bytes up to first NUL (which will be at copy_len unless input had embedded NUL).
        let nul = buf.iter().position(|&b| b == 0).unwrap_or(MAX_CITY_NAME);
        (buf, buf[..nul].to_vec())
    }

    fn add_node(&mut self, city_line: &str) -> Result<String, String> {
        if city_line.is_empty() {
            return Err("Error: NULL parameter in add_node".to_string());
        }
        if self.nodes.len() >= MAX_NODES {
            return Err(format!("Error: Graph is full (max {MAX_NODES} nodes)"));
        }

        let (name_buf, key) = Self::make_name_buf_from_input_line(city_line);
        if self.name_to_index.contains_key(&key) {
            let shown = String::from_utf8_lossy(&key).to_string();
            return Err(format!("Error: Node '{}' already exists", shown));
        }

        let idx = self.nodes.len();
        self.nodes.push(Node {
            name_buf,
            ref_count: 1,
            edges: Vec::new(),
        });
        self.name_to_index.insert(key.clone(), idx);

        Ok(String::from_utf8_lossy(&key).to_string())
    }

    fn get_node_by_name(&self, city_line: &str) -> Option<usize> {
        if city_line.is_empty() {
            return None;
        }
        let (_buf, key) = Self::make_name_buf_from_input_line(city_line);
        self.name_to_index.get(&key).copied()
    }

    fn add_edge(&mut self, from: usize, to: usize, distance: i32) -> Result<(), String> {
        if from >= self.nodes.len() || to >= self.nodes.len() {
            return Err("Error: NULL node in add_edge".to_string());
        }
        if self.nodes[from].edges.len() >= MAX_EDGES {
            let name = self.nodes[from].name_string_lossy();
            return Err(format!("Error: Node '{}' has maximum edges", name));
        }
        if distance < 0 {
            return Err("Error: Negative distance not allowed".to_string());
        }
        if self.nodes[from].edges.iter().any(|&(dst, _)| dst == to) {
            return Err("Error: Edge already exists".to_string());
        }
        self.nodes[from].edges.push((to, distance));
        Ok(())
    }

    fn shallow_copy(&mut self, start: usize) -> Result<(), String> {
        if start >= self.nodes.len() {
            return Err("Error: NULL node in shallow_copy".to_string());
        }

        // Increment refs recursively with visited set.
        let mut visited = vec![false; self.nodes.len().min(MAX_NODES)];
        let mut q = VecDeque::new();
        q.push_back(start);

        while let Some(u) = q.pop_front() {
            if u >= visited.len() || visited[u] {
                continue;
            }
            visited[u] = true;
            self.nodes[u].ref_count += 1;
            for &(v, _) in &self.nodes[u].edges {
                q.push_back(v);
            }
        }
        Ok(())
    }

    fn delete_node(&mut self, idx: usize) -> Result<(), String> {
        if idx >= self.nodes.len() {
            return Err("Error: NULL node in delete_node".to_string());
        }
        self.nodes[idx].ref_count -= 1;
        Ok(())
    }

    fn find_shortest_path(&self, start: usize, end: usize) -> Result<Vec<usize>, String> {
        if start >= self.nodes.len() || end >= self.nodes.len() {
            return Err("Error: NULL parameter in find_shortest_path".to_string());
        }

        let n = self.nodes.len().min(MAX_NODES);
        let mut dist = vec![INT_MAX; n];
        let mut prev: Vec<Option<usize>> = vec![None; n];
        let mut visited = vec![false; n];

        dist[start] = 0;

        loop {
            let mut current: Option<usize> = None;
            let mut min_d = INT_MAX;
            for i in 0..n {
                if !visited[i] && dist[i] < min_d {
                    min_d = dist[i];
                    current = Some(i);
                }
            }
            let Some(u) = current else { break };
            visited[u] = true;
            if u == end {
                break;
            }

            for &(v, w) in &self.nodes[u].edges {
                if v >= n {
                    continue;
                }
                let nd = dist[u].saturating_add(w);
                if nd < dist[v] {
                    dist[v] = nd;
                    prev[v] = Some(u);
                }
            }
        }

        if end >= n || dist[end] == INT_MAX {
            return Err("No path found".to_string());
        }

        let mut path_rev = Vec::new();
        let mut cur = Some(end);
        while let Some(u) = cur {
            path_rev.push(u);
            cur = prev[u];
        }
        path_rev.reverse();
        Ok(path_rev)
    }

    fn print_node(&self, idx: usize) {
        if idx >= self.nodes.len() {
            println!("NULL node");
            return;
        }
        let node = &self.nodes[idx];
        println!("City: {} (ref_count: {})", node.name_string_lossy(), node.ref_count);
        println!("  Edges:");
        for &(dst, dist) in &node.edges {
            let dst_name = self
                .nodes
                .get(dst)
                .map(|n| n.name_string_lossy())
                .unwrap_or_else(|| "NULL".to_string());
            println!("    -> {} (distance: {})", dst_name, dist);
        }
    }

    fn print_graph(&self) {
        println!("Graph with {} nodes:", self.nodes.len());
        for i in 0..self.nodes.len() {
            self.print_node(i);
        }
    }
}

struct Input {
    bytes: Vec<u8>,
    pos: usize,
}

impl Input {
    fn new_from_stdin() -> Self {
        let mut buf = Vec::new();
        io::stdin().read_to_end(&mut buf).unwrap();
        Self { bytes: buf, pos: 0 }
    }

    fn read_line_raw(&mut self) -> Option<String> {
        if self.pos >= self.bytes.len() {
            return None;
        }
        let start = self.pos;
        while self.pos < self.bytes.len() {
            let b = self.bytes[self.pos];
            if b == b'\n' || b == b'\r' {
                break;
            }
            self.pos += 1;
        }
        let end = self.pos;
        while self.pos < self.bytes.len() {
            let b = self.bytes[self.pos];
            if b == b'\n' || b == b'\r' {
                self.pos += 1;
            } else {
                break;
            }
        }
        Some(String::from_utf8_lossy(&self.bytes[start..end]).to_string())
    }

    fn read_nonempty_line(&mut self) -> Option<String> {
        while let Some(line) = self.read_line_raw() {
            if !line.is_empty() {
                return Some(line);
            }
        }
        None
    }
}

fn menu_print() {
    print!(
        "\n=== DAG City Route Manager ===\n\
1. Add city (node)\n\
2. Add route (edge)\n\
3. Show all cities\n\
4. Show city details\n\
5. Find shortest path\n\
6. Make shallow copy of subsection\n\
7. Delete node\n\
8. Exit\n\
Choice: "
    );
}

fn main() {
    print!("City Route Management System\nCommands are read from stdin\n");

    let mut input = Input::new_from_stdin();
    let mut graph = Graph::new();

    loop {
        menu_print();

        let Some(choice_line) = input.read_nonempty_line() else {
            print!("Freeing graph and exiting...\n");
            break;
        };

        let choice: i32 = match choice_line.trim().parse() {
            Ok(v) => v,
            Err(_) => {
                print!("Invalid input\n");
                continue;
            }
        };

        match choice {
            1 => {
                print!("Enter city name: ");
                let name_line = input.read_line_raw().unwrap_or_default();
                match graph.add_node(&name_line) {
                    Ok(stored) => print!("Added city: {}\n", stored),
                    Err(msg) => eprint!("{msg}\n"),
                }
            }
            2 => {
                print!("Enter from city: ");
                let from_line = input.read_line_raw().unwrap_or_default();
                print!("Enter to city: ");
                let to_line = input.read_line_raw().unwrap_or_default();
                print!("Enter distance: ");
                let dist_line = input.read_line_raw().unwrap_or_default();

                let distance: i32 = match dist_line.trim().parse() {
                    Ok(v) => v,
                    Err(_) => {
                        print!("Invalid input\n");
                        continue;
                    }
                };

                let from = graph.get_node_by_name(&from_line);
                let to = graph.get_node_by_name(&to_line);
                match (from, to) {
                    (Some(f), Some(t)) => match graph.add_edge(f, t, distance) {
                        Ok(()) => {
                            let (_fb, fk) = Graph::make_name_buf_from_input_line(&from_line);
                            let (_tb, tk) = Graph::make_name_buf_from_input_line(&to_line);
                            let fs = String::from_utf8_lossy(&fk).to_string();
                            let ts = String::from_utf8_lossy(&tk).to_string();
                            print!("Added route: {} -> {} (distance: {})\n", fs, ts, distance);
                        }
                        Err(msg) => eprint!("{msg}\n"),
                    },
                    _ => eprint!("Error: One or both cities not found\n"),
                }
            }
            3 => {
                graph.print_graph();
            }
            4 => {
                print!("Enter city name: ");
                let name_line = input.read_line_raw().unwrap_or_default();
                if let Some(idx) = graph.get_node_by_name(&name_line) {
                    graph.print_node(idx);
                } else {
                    eprint!("Error: City not found\n");
                }
            }
            5 => {
                print!("Enter start city: ");
                let start_line = input.read_line_raw().unwrap_or_default();
                print!("Enter end city: ");
                let end_line = input.read_line_raw().unwrap_or_default();

                let start = graph.get_node_by_name(&start_line);
                let end = graph.get_node_by_name(&end_line);
                match (start, end) {
                    (Some(s), Some(e)) => match graph.find_shortest_path(s, e) {
                        Ok(path) => {
                            let mut total = 0i32;
                            for w in path.windows(2) {
                                let u = w[0];
                                let v = w[1];
                                if let Some((_, d)) =
                                    graph.nodes[u].edges.iter().find(|&&(dst, _)| dst == v)
                                {
                                    total = total.saturating_add(*d);
                                }
                            }
                            print!("Shortest path: ");
                            for (i, &idx) in path.iter().enumerate() {
                                if i != 0 {
                                    print!(" -> ");
                                }
                                print!("{}", graph.nodes[idx].name_string_lossy());
                            }
                            print!("\nTotal distance: {}\n", total);
                        }
                        Err(msg) => eprint!("{msg}\n"),
                    },
                    _ => eprint!("Error: One or both cities not found\n"),
                }
            }
            6 => {
                print!("Enter start city for shallow copy: ");
                let name_line = input.read_line_raw().unwrap_or_default();
                if let Some(idx) = graph.get_node_by_name(&name_line) {
                    match graph.shallow_copy(idx) {
                        Ok(()) => {
                            let (_b, k) = Graph::make_name_buf_from_input_line(&name_line);
                            let shown = String::from_utf8_lossy(&k).to_string();
                            print!("Shallow copy created from: {}\n", shown);
                        }
                        Err(msg) => eprint!("{msg}\n"),
                    }
                } else {
                    eprint!("Error: City not found\n");
                }
            }
            7 => {
                print!("Enter city name to delete: ");
                let name_line = input.read_line_raw().unwrap_or_default();
                if let Some(idx) = graph.get_node_by_name(&name_line) {
                    match graph.delete_node(idx) {
                        Ok(()) => {
                            let (_b, k) = Graph::make_name_buf_from_input_line(&name_line);
                            let shown = String::from_utf8_lossy(&k).to_string();
                            print!("Deleted node (decremented ref_count): {}\n", shown);
                        }
                        Err(msg) => eprint!("{msg}\n"),
                    }
                } else {
                    eprint!("Error: City not found\n");
                }
            }
            8 => {
                print!("Freeing graph and exiting...\n");
                break;
            }
            _ => {
                print!("Invalid input\n");
            }
        }
    }
}
```

**Entity:** Graph

**States:** ValidEdgeSet, EdgeAdditionRejected

**Transitions:**
- ValidEdgeSet -> ValidEdgeSet via add_edge() when checks pass
- ValidEdgeSet -> EdgeAdditionRejected via add_edge() when a precondition fails

**Evidence:** Constants: `const MAX_EDGES: usize = 10;` and Node field `edges: Vec<(usize, i32)>` (unconstrained container); Graph::add_edge(): `if self.nodes[from].edges.len() >= MAX_EDGES { ... Err(format!("Error: Node '{}' has maximum edges", name)) }`; Graph::add_edge(): `if distance < 0 { return Err("Error: Negative distance not allowed".to_string()); }`; Graph::add_edge(): duplicate prevention `if self.nodes[from].edges.iter().any(|&(dst, _)| dst == to) { return Err("Error: Edge already exists".to_string()); }`

**Implementation:** Use `NonNegativeDistance(i32)` (constructible only via a checked constructor) to eliminate the `distance < 0` branch. For uniqueness/degree bounds, wrap `edges` in a dedicated `OutgoingEdges` type that internally uses `HashMap<NodeId, NonNegativeDistance>` (enforces uniqueness) and a const-generic capacity (or a checked `push_edge` API) to enforce MAX_EDGES at the API boundary rather than scattering checks.

---

## Protocol Invariants

### 2. Node ref_count validity (non-negative, consistent with sharing protocol)

**Location**: `/data/test_case/main.rs:1-22`

**Confidence**: medium

**Suggested Pattern**: raii

**Description**: Node includes a manual reference counter (ref_count: i32) implying an ownership/sharing protocol (increment/decrement on clone/retain/release) with an implicit invariant that the count should never be negative and should track the number of outstanding references. This is not enforced by the type system: any code can set ref_count arbitrarily, and Clone is derived for Node without any visible adjustment to ref_count, suggesting a potential mismatch between cloning and the intended refcounting protocol.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Graph, impl Graph (10 methods); struct Input, impl Input (3 methods); 2 free function(s)

const INT_MAX: i32 = i32::MAX;

#[derive(Clone)]
struct Node {
    // Store as raw bytes (C-like), may include any bytes except we keep it as entered (minus newline),
    // and comparisons/printing use C-string semantics (stop at first NUL).
    name_buf: [u8; MAX_CITY_NAME], // always NUL-terminated somewhere
    ref_count: i32,
    edges: Vec<(usize, i32)>, // (destination index, distance)
}

impl Node {
    fn name_cstr_bytes(&self) -> &[u8] {
        let nul = self.name_buf.iter().position(|&b| b == 0).unwrap_or(MAX_CITY_NAME);
        &self.name_buf[..nul]
    }
    fn name_string_lossy(&self) -> String {
        String::from_utf8_lossy(self.name_cstr_bytes()).to_string()
    }
}

```

**Entity:** Node

**States:** NonNegativeRefCount, NegativeOrInconsistentRefCount

**Transitions:**
- NonNegativeRefCount -> NegativeOrInconsistentRefCount via arbitrary mutation or cloning without updating ref_count (no enforcing API shown)
- NegativeOrInconsistentRefCount -> NonNegativeRefCount via manual correction (not represented in types/methods here)

**Evidence:** derive: #[derive(Clone)] on Node; field: ref_count: i32 (manual refcount encoded as a plain integer); absence of any refcount-management methods in impl Node (only name accessors), meaning the protocol is external and unenforced

**Implementation:** Eliminate manual ref_count and use Rc/Arc (possibly with Weak) to encode shared ownership, or encapsulate refcount changes behind RAII guard types (retain/release) so increments/decrements are tied to lifetimes and cannot be forgotten; if Node must be clonable, use Rc<NodeInner> and clone the Rc.

---

### 3. Graph node-index validity & ownership protocol (Name -> NodeId; indices must be in-bounds and refer to existing nodes)

**Location**: `/data/test_case/main.rs:1-197`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: Graph’s public-facing operations are implicitly defined over a set of valid node identifiers (indices into `nodes`) that must come from this same `Graph` and be in-bounds at the time of use. This is currently enforced by pervasive runtime checks (`if idx >= self.nodes.len()`) and by returning `Option<usize>`/`Result<_, String>` rather than making invalid indices unrepresentable. Additionally, `name_to_index` must stay consistent with `nodes` (the map values must point at the correct slot for the keyed name); this relies on the convention that nodes are only appended and never removed/reordered (since deletion only decrements `ref_count`). None of these constraints are expressed in the type system because node IDs are plain `usize` and name keys are plain `Vec<u8>`.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Node, impl Node (2 methods); struct Input, impl Input (3 methods); 2 free function(s)

    }
}

struct Graph {
    nodes: Vec<Node>,
    // Keyed by C-string bytes (up to first NUL) exactly like strcmp would see.
    name_to_index: HashMap<Vec<u8>, usize>,
}

impl Graph {
    fn new() -> Self {
        Self {
            nodes: Vec::new(),
            name_to_index: HashMap::new(),
        }
    }

    fn make_name_buf_from_input_line(line: &str) -> ([u8; MAX_CITY_NAME], Vec<u8>) {
        // C code copies exactly MAX_CITY_NAME-1 bytes from the provided pointer,
        // then sets last byte to NUL. That means:
        // - If input is longer than 63 bytes, stored name is first 63 bytes.
        // - If input is shorter, it still copies bytes beyond end in C (UB),
        //   but in the interactive program this would be a proper C string.
        // For our executable, we treat the line as the C string content and copy up to 63 bytes.
        let mut buf = [0u8; MAX_CITY_NAME];
        let bytes = line.as_bytes();
        let copy_len = bytes.len().min(MAX_CITY_NAME - 1);
        buf[..copy_len].copy_from_slice(&bytes[..copy_len]);
        buf[MAX_CITY_NAME - 1] = 0;

        // Key is bytes up to first NUL (which will be at copy_len unless input had embedded NUL).
        let nul = buf.iter().position(|&b| b == 0).unwrap_or(MAX_CITY_NAME);
        (buf, buf[..nul].to_vec())
    }

    fn add_node(&mut self, city_line: &str) -> Result<String, String> {
        if city_line.is_empty() {
            return Err("Error: NULL parameter in add_node".to_string());
        }
        if self.nodes.len() >= MAX_NODES {
            return Err(format!("Error: Graph is full (max {MAX_NODES} nodes)"));
        }

        let (name_buf, key) = Self::make_name_buf_from_input_line(city_line);
        if self.name_to_index.contains_key(&key) {
            let shown = String::from_utf8_lossy(&key).to_string();
            return Err(format!("Error: Node '{}' already exists", shown));
        }

        let idx = self.nodes.len();
        self.nodes.push(Node {
            name_buf,
            ref_count: 1,
            edges: Vec::new(),
        });
        self.name_to_index.insert(key.clone(), idx);

        Ok(String::from_utf8_lossy(&key).to_string())
    }

    fn get_node_by_name(&self, city_line: &str) -> Option<usize> {
        if city_line.is_empty() {
            return None;
        }
        let (_buf, key) = Self::make_name_buf_from_input_line(city_line);
        self.name_to_index.get(&key).copied()
    }

    fn add_edge(&mut self, from: usize, to: usize, distance: i32) -> Result<(), String> {
        if from >= self.nodes.len() || to >= self.nodes.len() {
            return Err("Error: NULL node in add_edge".to_string());
        }
        if self.nodes[from].edges.len() >= MAX_EDGES {
            let name = self.nodes[from].name_string_lossy();
            return Err(format!("Error: Node '{}' has maximum edges", name));
        }
        if distance < 0 {
            return Err("Error: Negative distance not allowed".to_string());
        }
        if self.nodes[from].edges.iter().any(|&(dst, _)| dst == to) {
            return Err("Error: Edge already exists".to_string());
        }
        self.nodes[from].edges.push((to, distance));
        Ok(())
    }

    fn shallow_copy(&mut self, start: usize) -> Result<(), String> {
        if start >= self.nodes.len() {
            return Err("Error: NULL node in shallow_copy".to_string());
        }

        // Increment refs recursively with visited set.
        let mut visited = vec![false; self.nodes.len().min(MAX_NODES)];
        let mut q = VecDeque::new();
        q.push_back(start);

        while let Some(u) = q.pop_front() {
            if u >= visited.len() || visited[u] {
                continue;
            }
            visited[u] = true;
            self.nodes[u].ref_count += 1;
            for &(v, _) in &self.nodes[u].edges {
                q.push_back(v);
            }
        }
        Ok(())
    }

    fn delete_node(&mut self, idx: usize) -> Result<(), String> {
        if idx >= self.nodes.len() {
            return Err("Error: NULL node in delete_node".to_string());
        }
        self.nodes[idx].ref_count -= 1;
        Ok(())
    }

    fn find_shortest_path(&self, start: usize, end: usize) -> Result<Vec<usize>, String> {
        if start >= self.nodes.len() || end >= self.nodes.len() {
            return Err("Error: NULL parameter in find_shortest_path".to_string());
        }

        let n = self.nodes.len().min(MAX_NODES);
        let mut dist = vec![INT_MAX; n];
        let mut prev: Vec<Option<usize>> = vec![None; n];
        let mut visited = vec![false; n];

        dist[start] = 0;

        loop {
            let mut current: Option<usize> = None;
            let mut min_d = INT_MAX;
            for i in 0..n {
                if !visited[i] && dist[i] < min_d {
                    min_d = dist[i];
                    current = Some(i);
                }
            }
            let Some(u) = current else { break };
            visited[u] = true;
            if u == end {
                break;
            }

            for &(v, w) in &self.nodes[u].edges {
                if v >= n {
                    continue;
                }
                let nd = dist[u].saturating_add(w);
                if nd < dist[v] {
                    dist[v] = nd;
                    prev[v] = Some(u);
                }
            }
        }

        if end >= n || dist[end] == INT_MAX {
            return Err("No path found".to_string());
        }

        let mut path_rev = Vec::new();
        let mut cur = Some(end);
        while let Some(u) = cur {
            path_rev.push(u);
            cur = prev[u];
        }
        path_rev.reverse();
        Ok(path_rev)
    }

    fn print_node(&self, idx: usize) {
        if idx >= self.nodes.len() {
            println!("NULL node");
            return;
        }
        let node = &self.nodes[idx];
        println!("City: {} (ref_count: {})", node.name_string_lossy(), node.ref_count);
        println!("  Edges:");
        for &(dst, dist) in &node.edges {
            let dst_name = self
                .nodes
                .get(dst)
                .map(|n| n.name_string_lossy())
                .unwrap_or_else(|| "NULL".to_string());
            println!("    -> {} (distance: {})", dst_name, dist);
        }
    }

    fn print_graph(&self) {
        println!("Graph with {} nodes:", self.nodes.len());
        for i in 0..self.nodes.len() {
            self.print_node(i);
        }
    }
}

```

**Entity:** Graph

**States:** HasOnlyValidNodeIds, MayContainInvalid/ForeignIndices

**Transitions:**
- MayContainInvalid/ForeignIndices -> HasOnlyValidNodeIds via using a graph-specific NodeId newtype returned from add_node/get_node_by_name (eliminates foreign/out-of-bounds indices)

**Evidence:** field: `nodes: Vec<Node>` is indexed by `usize` throughout the API; field: `name_to_index: HashMap<Vec<u8>, usize>` stores indices as raw `usize` tied to `nodes` layout; method `add_edge(&mut self, from: usize, to: usize, ...)`: `if from >= self.nodes.len() || to >= self.nodes.len() { return Err("Error: NULL node in add_edge".to_string()); }`; method `shallow_copy(&mut self, start: usize)`: `if start >= self.nodes.len() { return Err("Error: NULL node in shallow_copy".to_string()); }`; method `delete_node(&mut self, idx: usize)`: `if idx >= self.nodes.len() { return Err("Error: NULL node in delete_node".to_string()); }`; method `find_shortest_path(&self, start: usize, end: usize)`: `if start >= self.nodes.len() || end >= self.nodes.len() { return Err("Error: NULL parameter in find_shortest_path".to_string()); }`; method `print_node(&self, idx: usize)`: `if idx >= self.nodes.len() { println!("NULL node"); return; }`; method `print_node`: edges store `dst` as a raw index and must be checked at use: `.nodes.get(dst)...unwrap_or_else(|| "NULL".to_string())`

**Implementation:** Introduce `struct NodeId(usize);` (optionally with a lifetime: `struct NodeId<'g>(usize, PhantomData<&'g Graph>)`). Change APIs to return/accept `NodeId` instead of `usize`: `add_node(&mut self, ...) -> Result<NodeId, _>`, `get_node_by_name(&self, ...) -> Option<NodeId>`, `add_edge(&mut self, from: NodeId, to: NodeId, ...)`. Keep bounds checks inside constructors/conversions only, making it impossible to call graph methods with arbitrary `usize`.

---

### 7. Graph name/index coherence + stable node-index handle protocol

**Location**: `/data/test_case/main.rs:1-445`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: Graph exposes and internally relies on a protocol where node identity is a stable `usize` index into `nodes`, and `name_to_index` must remain a total, coherent mapping from a canonicalized name key (C-string bytes up to first NUL after truncation) to that index. Many operations require that indices are valid for the current `nodes` vector, and that all name lookups/canonicalization are performed consistently via `make_name_buf_from_input_line`. None of this is enforced by the type system: callers can pass arbitrary `usize` values to `add_edge`, `shallow_copy`, `delete_node`, `find_shortest_path`, and the internal invariant that `name_to_index[key] < nodes.len()` and refers to the node whose `name_buf` corresponds to `key` is maintained by convention rather than types.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Node, impl Node (2 methods); struct Graph, impl Graph (10 methods); struct Input, impl Input (3 methods)

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![feature(c_variadic)]
#![feature(extern_types)]
#![feature(linkage)]
#![feature(rustc_private)]
#![feature(thread_local)]
#![feature(formatting_options)]

use std::collections::{HashMap, VecDeque};
use std::io::{self, Read};

const MAX_CITY_NAME: usize = 64; // C buffer size, includes NUL
const MAX_EDGES: usize = 10;
const MAX_NODES: usize = 100;
const INT_MAX: i32 = i32::MAX;

#[derive(Clone)]
struct Node {
    // Store as raw bytes (C-like), may include any bytes except we keep it as entered (minus newline),
    // and comparisons/printing use C-string semantics (stop at first NUL).
    name_buf: [u8; MAX_CITY_NAME], // always NUL-terminated somewhere
    ref_count: i32,
    edges: Vec<(usize, i32)>, // (destination index, distance)
}

impl Node {
    fn name_cstr_bytes(&self) -> &[u8] {
        let nul = self.name_buf.iter().position(|&b| b == 0).unwrap_or(MAX_CITY_NAME);
        &self.name_buf[..nul]
    }
    fn name_string_lossy(&self) -> String {
        String::from_utf8_lossy(self.name_cstr_bytes()).to_string()
    }
}

struct Graph {
    nodes: Vec<Node>,
    // Keyed by C-string bytes (up to first NUL) exactly like strcmp would see.
    name_to_index: HashMap<Vec<u8>, usize>,
}

impl Graph {
    fn new() -> Self {
        Self {
            nodes: Vec::new(),
            name_to_index: HashMap::new(),
        }
    }

    fn make_name_buf_from_input_line(line: &str) -> ([u8; MAX_CITY_NAME], Vec<u8>) {
        // C code copies exactly MAX_CITY_NAME-1 bytes from the provided pointer,
        // then sets last byte to NUL. That means:
        // - If input is longer than 63 bytes, stored name is first 63 bytes.
        // - If input is shorter, it still copies bytes beyond end in C (UB),
        //   but in the interactive program this would be a proper C string.
        // For our executable, we treat the line as the C string content and copy up to 63 bytes.
        let mut buf = [0u8; MAX_CITY_NAME];
        let bytes = line.as_bytes();
        let copy_len = bytes.len().min(MAX_CITY_NAME - 1);
        buf[..copy_len].copy_from_slice(&bytes[..copy_len]);
        buf[MAX_CITY_NAME - 1] = 0;

        // Key is bytes up to first NUL (which will be at copy_len unless input had embedded NUL).
        let nul = buf.iter().position(|&b| b == 0).unwrap_or(MAX_CITY_NAME);
        (buf, buf[..nul].to_vec())
    }

    fn add_node(&mut self, city_line: &str) -> Result<String, String> {
        if city_line.is_empty() {
            return Err("Error: NULL parameter in add_node".to_string());
        }
        if self.nodes.len() >= MAX_NODES {
            return Err(format!("Error: Graph is full (max {MAX_NODES} nodes)"));
        }

        let (name_buf, key) = Self::make_name_buf_from_input_line(city_line);
        if self.name_to_index.contains_key(&key) {
            let shown = String::from_utf8_lossy(&key).to_string();
            return Err(format!("Error: Node '{}' already exists", shown));
        }

        let idx = self.nodes.len();
        self.nodes.push(Node {
            name_buf,
            ref_count: 1,
            edges: Vec::new(),
        });
        self.name_to_index.insert(key.clone(), idx);

        Ok(String::from_utf8_lossy(&key).to_string())
    }

    fn get_node_by_name(&self, city_line: &str) -> Option<usize> {
        if city_line.is_empty() {
            return None;
        }
        let (_buf, key) = Self::make_name_buf_from_input_line(city_line);
        self.name_to_index.get(&key).copied()
    }

    fn add_edge(&mut self, from: usize, to: usize, distance: i32) -> Result<(), String> {
        if from >= self.nodes.len() || to >= self.nodes.len() {
            return Err("Error: NULL node in add_edge".to_string());
        }
        if self.nodes[from].edges.len() >= MAX_EDGES {
            let name = self.nodes[from].name_string_lossy();
            return Err(format!("Error: Node '{}' has maximum edges", name));
        }
        if distance < 0 {
            return Err("Error: Negative distance not allowed".to_string());
        }
        if self.nodes[from].edges.iter().any(|&(dst, _)| dst == to) {
            return Err("Error: Edge already exists".to_string());
        }
        self.nodes[from].edges.push((to, distance));
        Ok(())
    }

    fn shallow_copy(&mut self, start: usize) -> Result<(), String> {
        if start >= self.nodes.len() {
            return Err("Error: NULL node in shallow_copy".to_string());
        }

        // Increment refs recursively with visited set.
        let mut visited = vec![false; self.nodes.len().min(MAX_NODES)];
        let mut q = VecDeque::new();
        q.push_back(start);

        while let Some(u) = q.pop_front() {
            if u >= visited.len() || visited[u] {
                continue;
            }
            visited[u] = true;
            self.nodes[u].ref_count += 1;
            for &(v, _) in &self.nodes[u].edges {
                q.push_back(v);
            }
        }
        Ok(())
    }

    fn delete_node(&mut self, idx: usize) -> Result<(), String> {
        if idx >= self.nodes.len() {
            return Err("Error: NULL node in delete_node".to_string());
        }
        self.nodes[idx].ref_count -= 1;
        Ok(())
    }

    fn find_shortest_path(&self, start: usize, end: usize) -> Result<Vec<usize>, String> {
        if start >= self.nodes.len() || end >= self.nodes.len() {
            return Err("Error: NULL parameter in find_shortest_path".to_string());
        }

        let n = self.nodes.len().min(MAX_NODES);
        let mut dist = vec![INT_MAX; n];
        let mut prev: Vec<Option<usize>> = vec![None; n];
        let mut visited = vec![false; n];

        dist[start] = 0;

        loop {
            let mut current: Option<usize> = None;
            let mut min_d = INT_MAX;
            for i in 0..n {
                if !visited[i] && dist[i] < min_d {
                    min_d = dist[i];
                    current = Some(i);
                }
            }
            let Some(u) = current else { break };
            visited[u] = true;
            if u == end {
                break;
            }

            for &(v, w) in &self.nodes[u].edges {
                if v >= n {
                    continue;
                }
                let nd = dist[u].saturating_add(w);
                if nd < dist[v] {
                    dist[v] = nd;
                    prev[v] = Some(u);
                }
            }
        }

        if end >= n || dist[end] == INT_MAX {
            return Err("No path found".to_string());
        }

        let mut path_rev = Vec::new();
        let mut cur = Some(end);
        while let Some(u) = cur {
            path_rev.push(u);
            cur = prev[u];
        }
        path_rev.reverse();
        Ok(path_rev)
    }

    fn print_node(&self, idx: usize) {
        if idx >= self.nodes.len() {
            println!("NULL node");
            return;
        }
        let node = &self.nodes[idx];
        println!("City: {} (ref_count: {})", node.name_string_lossy(), node.ref_count);
        println!("  Edges:");
        for &(dst, dist) in &node.edges {
            let dst_name = self
                .nodes
                .get(dst)
                .map(|n| n.name_string_lossy())
                .unwrap_or_else(|| "NULL".to_string());
            println!("    -> {} (distance: {})", dst_name, dist);
        }
    }

    fn print_graph(&self) {
        println!("Graph with {} nodes:", self.nodes.len());
        for i in 0..self.nodes.len() {
            self.print_node(i);
        }
    }
}

struct Input {
    bytes: Vec<u8>,
    pos: usize,
}

impl Input {
    fn new_from_stdin() -> Self {
        let mut buf = Vec::new();
        io::stdin().read_to_end(&mut buf).unwrap();
        Self { bytes: buf, pos: 0 }
    }

    fn read_line_raw(&mut self) -> Option<String> {
        if self.pos >= self.bytes.len() {
            return None;
        }
        let start = self.pos;
        while self.pos < self.bytes.len() {
            let b = self.bytes[self.pos];
            if b == b'\n' || b == b'\r' {
                break;
            }
            self.pos += 1;
        }
        let end = self.pos;
        while self.pos < self.bytes.len() {
            let b = self.bytes[self.pos];
            if b == b'\n' || b == b'\r' {
                self.pos += 1;
            } else {
                break;
            }
        }
        Some(String::from_utf8_lossy(&self.bytes[start..end]).to_string())
    }

    fn read_nonempty_line(&mut self) -> Option<String> {
        while let Some(line) = self.read_line_raw() {
            if !line.is_empty() {
                return Some(line);
            }
        }
        None
    }
}

fn menu_print() {
    print!(
        "\n=== DAG City Route Manager ===\n\
1. Add city (node)\n\
2. Add route (edge)\n\
3. Show all cities\n\
4. Show city details\n\
5. Find shortest path\n\
6. Make shallow copy of subsection\n\
7. Delete node\n\
8. Exit\n\
Choice: "
    );
}

fn main() {
    print!("City Route Management System\nCommands are read from stdin\n");

    let mut input = Input::new_from_stdin();
    let mut graph = Graph::new();

    loop {
        menu_print();

        let Some(choice_line) = input.read_nonempty_line() else {
            print!("Freeing graph and exiting...\n");
            break;
        };

        let choice: i32 = match choice_line.trim().parse() {
            Ok(v) => v,
            Err(_) => {
                print!("Invalid input\n");
                continue;
            }
        };

        match choice {
            1 => {
                print!("Enter city name: ");
                let name_line = input.read_line_raw().unwrap_or_default();
                match graph.add_node(&name_line) {
                    Ok(stored) => print!("Added city: {}\n", stored),
                    Err(msg) => eprint!("{msg}\n"),
                }
            }
            2 => {
                print!("Enter from city: ");
                let from_line = input.read_line_raw().unwrap_or_default();
                print!("Enter to city: ");
                let to_line = input.read_line_raw().unwrap_or_default();
                print!("Enter distance: ");
                let dist_line = input.read_line_raw().unwrap_or_default();

                let distance: i32 = match dist_line.trim().parse() {
                    Ok(v) => v,
                    Err(_) => {
                        print!("Invalid input\n");
                        continue;
                    }
                };

                let from = graph.get_node_by_name(&from_line);
                let to = graph.get_node_by_name(&to_line);
                match (from, to) {
                    (Some(f), Some(t)) => match graph.add_edge(f, t, distance) {
                        Ok(()) => {
                            let (_fb, fk) = Graph::make_name_buf_from_input_line(&from_line);
                            let (_tb, tk) = Graph::make_name_buf_from_input_line(&to_line);
                            let fs = String::from_utf8_lossy(&fk).to_string();
                            let ts = String::from_utf8_lossy(&tk).to_string();
                            print!("Added route: {} -> {} (distance: {})\n", fs, ts, distance);
                        }
                        Err(msg) => eprint!("{msg}\n"),
                    },
                    _ => eprint!("Error: One or both cities not found\n"),
                }
            }
            3 => {
                graph.print_graph();
            }
            4 => {
                print!("Enter city name: ");
                let name_line = input.read_line_raw().unwrap_or_default();
                if let Some(idx) = graph.get_node_by_name(&name_line) {
                    graph.print_node(idx);
                } else {
                    eprint!("Error: City not found\n");
                }
            }
            5 => {
                print!("Enter start city: ");
                let start_line = input.read_line_raw().unwrap_or_default();
                print!("Enter end city: ");
                let end_line = input.read_line_raw().unwrap_or_default();

                let start = graph.get_node_by_name(&start_line);
                let end = graph.get_node_by_name(&end_line);
                match (start, end) {
                    (Some(s), Some(e)) => match graph.find_shortest_path(s, e) {
                        Ok(path) => {
                            let mut total = 0i32;
                            for w in path.windows(2) {
                                let u = w[0];
                                let v = w[1];
                                if let Some((_, d)) =
                                    graph.nodes[u].edges.iter().find(|&&(dst, _)| dst == v)
                                {
                                    total = total.saturating_add(*d);
                                }
                            }
                            print!("Shortest path: ");
                            for (i, &idx) in path.iter().enumerate() {
                                if i != 0 {
                                    print!(" -> ");
                                }
                                print!("{}", graph.nodes[idx].name_string_lossy());
                            }
                            print!("\nTotal distance: {}\n", total);
                        }
                        Err(msg) => eprint!("{msg}\n"),
                    },
                    _ => eprint!("Error: One or both cities not found\n"),
                }
            }
            6 => {
                print!("Enter start city for shallow copy: ");
                let name_line = input.read_line_raw().unwrap_or_default();
                if let Some(idx) = graph.get_node_by_name(&name_line) {
                    match graph.shallow_copy(idx) {
                        Ok(()) => {
                            let (_b, k) = Graph::make_name_buf_from_input_line(&name_line);
                            let shown = String::from_utf8_lossy(&k).to_string();
                            print!("Shallow copy created from: {}\n", shown);
                        }
                        Err(msg) => eprint!("{msg}\n"),
                    }
                } else {
                    eprint!("Error: City not found\n");
                }
            }
            7 => {
                print!("Enter city name to delete: ");
                let name_line = input.read_line_raw().unwrap_or_default();
                if let Some(idx) = graph.get_node_by_name(&name_line) {
                    match graph.delete_node(idx) {
                        Ok(()) => {
                            let (_b, k) = Graph::make_name_buf_from_input_line(&name_line);
                            let shown = String::from_utf8_lossy(&k).to_string();
                            print!("Deleted node (decremented ref_count): {}\n", shown);
                        }
                        Err(msg) => eprint!("{msg}\n"),
                    }
                } else {
                    eprint!("Error: City not found\n");
                }
            }
            8 => {
                print!("Freeing graph and exiting...\n");
                break;
            }
            _ => {
                print!("Invalid input\n");
            }
        }
    }
}
```

**Entity:** Graph

**States:** Empty, HasNodes

**Transitions:**
- Empty -> HasNodes via add_node()

**Evidence:** Graph fields: `nodes: Vec<Node>` and `name_to_index: HashMap<Vec<u8>, usize>` encode identity as raw indices; Graph::add_node(): inserts `idx = self.nodes.len()` into both `nodes.push(...)` and `name_to_index.insert(key.clone(), idx)`; relies on them staying in sync; Graph::get_node_by_name(): recomputes key using `make_name_buf_from_input_line` and returns `usize` index (a raw handle) via `copied()`; Graph::add_edge(from: usize, to: usize, ...): runtime check `if from >= self.nodes.len() || to >= self.nodes.len() { return Err("Error: NULL node in add_edge".to_string()); }` shows the precondition is 'indices must be valid'; Graph::find_shortest_path(start: usize, end: usize): same index precondition enforced by `if start >= self.nodes.len() || end >= self.nodes.len() { return Err("Error: NULL parameter in find_shortest_path".to_string()); }`

**Implementation:** Introduce a `NodeId(usize)` newtype (kept private) and only construct it through `Graph::add_node`/`Graph::get_node_by_name`. Change APIs to accept `NodeId` instead of `usize`. Optionally also make `Graph::nodes` private (it is currently accessed directly in `main`) and provide safe accessors that take `NodeId`. This prevents arbitrary indices from being passed and centralizes the name-key canonicalization path.

---


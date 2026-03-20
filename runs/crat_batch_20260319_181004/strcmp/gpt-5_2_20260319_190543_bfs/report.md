# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 5
- **Temporal ordering**: 0
- **Resource lifecycle**: 0
- **State machine**: 3
- **Precondition**: 1
- **Protocol**: 1
- **Modules analyzed**: 1

## State Machine Invariants

### 1. User authentication state (LoggedOut / LoggedIn) gated by logged_in flag

**Location**: `/data/test_case/main.rs:1-11`

**Confidence**: low

**Suggested Pattern**: typestate

**Description**: User has an implicit authentication state tracked at runtime by the `logged_in: bool` field. The presence of both credentials (`name`, `password`) and a privilege indicator (`permission_level`) alongside `logged_in` suggests that some operations are only valid when the user is authenticated (LoggedIn), and others (like authentication) transition from LoggedOut to LoggedIn. None of these constraints are enforced by the type system because `User` is a single type whose fields are always accessible regardless of whether `logged_in` is true/false.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct FileEntry; struct Variable; struct State, 19 free function(s); 13 free function(s)

const MAX_VARIABLES: usize = 20;

#[derive(Clone, Default)]
struct User {
    name: Vec<u8>,
    password: Vec<u8>,
    permission_level: i32,
    logged_in: bool,
}

```

**Entity:** User

**States:** LoggedOut, LoggedIn

**Transitions:**
- LoggedOut -> LoggedIn via (not shown) login/authenticate operation
- LoggedIn -> LoggedOut via (not shown) logout operation

**Evidence:** line 11: `logged_in: bool` encodes authentication state at runtime; line 10: `permission_level: i32` indicates authorization/privilege that likely should be meaningful only when authenticated; line 8-9: `name: Vec<u8>`, `password: Vec<u8>` stored on the same struct as `logged_in`, implying credential-bearing identity with a login state

**Implementation:** Model authentication as `User<S>` with zero-sized states `LoggedOut` and `LoggedIn`. Keep only unauthenticated-safe fields on `User<LoggedOut>`. Provide `fn login(self, creds: ...) -> Result<User<LoggedIn>, _>` consuming transition, and `fn logout(self) -> User<LoggedOut>`. Expose privileged operations only on `User<LoggedIn>`.

---

### 2. State session protocol (No current user / User selected) with index validity invariants

**Location**: `/data/test_case/main.rs:1-13`

**Confidence**: low

**Suggested Pattern**: typestate

**Description**: State encodes a session-like notion of whether a 'current user' is selected via `current_user: Option<usize>`. This creates an implicit state machine: when `current_user` is `None`, any operation that assumes a selected user must be forbidden or must error; when it is `Some(i)`, `i` must be a valid index into `users` and must continue to refer to the intended user across mutations (e.g., insert/remove/reorder). The type system does not enforce (1) that a user is selected before user-dependent operations, nor (2) that the stored index remains in-bounds and stable with respect to `users` changes.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct User; struct FileEntry; struct Variable; 13 free function(s)

}

#[derive(Default)]
struct State {
    users: Vec<User>,
    current_user: Option<usize>,
    files: Vec<FileEntry>,
    variables: Vec<Variable>,
    debug_mode: bool,
    verbose_mode: bool,
}

```

**Entity:** State

**States:** NoCurrentUser, CurrentUserSelected

**Transitions:**
- NoCurrentUser -> CurrentUserSelected via setting `current_user = Some(i)`
- CurrentUserSelected -> NoCurrentUser via setting `current_user = None`
- CurrentUserSelected -> CurrentUserSelected via changing `current_user` to a different `Some(i)`

**Evidence:** line 10: `current_user: Option<usize>` encodes whether a current user is selected; line 9: `users: Vec<User>` combined with `Option<usize>` implies an index-into-vec invariant: if `Some(i)`, then `i < users.len()`

**Implementation:** Split into `State<NoUser>` and `State<HasUser>` (or `State` plus a `CurrentUser<'a>` capability). Provide an operation that selects a user and returns a typed handle/capability (e.g., `fn select_user(&mut self, idx: UserId) -> CurrentUser<'_>`), where `CurrentUser` carries a borrow into `State` and grants access to user-dependent actions. Replace raw `usize` with a newtype `UserId` and/or a generational index (e.g., slotmap) to make references stable across `Vec` mutations.

---

### 3. Authentication session protocol (LoggedOut / LoggedIn user)

**Location**: `/data/test_case/main.rs:1-584`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: State encodes an authentication session using `current_user: Option<usize>` plus `User.logged_in: bool`. Many commands require a logged-in user and do runtime checks via `require_logged_in()` and ad-hoc checks in commands. The type system does not prevent calling "requires auth" operations when logged out, nor does it guarantee that `current_user` is `Some(i)` implies `users[i].logged_in == true` and `i` is in-bounds. This leads to scattered checks and even an `unwrap()` that assumes the invariant after a separate check.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct User; struct FileEntry; struct Variable; struct State, 19 free function(s)

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![feature(c_variadic)]
#![feature(extern_types)]
#![feature(linkage)]
#![feature(rustc_private)]
#![feature(thread_local)]
#![feature(formatting_options)]
#![feature(as_array_of_cells)]

use std::io::{self, Write};

const MAX_INPUT: usize = 256;
const MAX_COMMAND: usize = 64;
const MAX_ARGS: usize = 10;
const MAX_FILES: usize = 20;
const MAX_USERS: usize = 10;
const MAX_VARIABLES: usize = 20;

#[derive(Clone, Default)]
struct User {
    name: Vec<u8>,
    password: Vec<u8>,
    permission_level: i32,
    logged_in: bool,
}

#[derive(Clone, Default)]
struct FileEntry {
    filename: Vec<u8>,
    content: Vec<u8>,
    owner: Vec<u8>,
    permissions: i32,
}

#[derive(Clone, Default)]
struct Variable {
    name: Vec<u8>,
    value: Vec<u8>,
}

#[derive(Default)]
struct State {
    users: Vec<User>,
    current_user: Option<usize>,
    files: Vec<FileEntry>,
    variables: Vec<Variable>,
    debug_mode: bool,
    verbose_mode: bool,
}

fn bytes_to_lossy_string(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

// C-like atoi: parses optional whitespace, optional sign, then digits; stops at first non-digit.
// Uses wrapping arithmetic to mimic typical C overflow behavior.
fn c_atoi_wrapping(s: &[u8]) -> i32 {
    let mut i = 0usize;
    while i < s.len() && matches!(s[i], b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c) {
        i += 1;
    }
    let mut sign: i32 = 1;
    if i < s.len() {
        if s[i] == b'-' {
            sign = -1;
            i += 1;
        } else if s[i] == b'+' {
            i += 1;
        }
    }
    let mut acc: i32 = 0;
    while i < s.len() {
        let c = s[i];
        if !(b'0'..=b'9').contains(&c) {
            break;
        }
        let digit = (c - b'0') as i32;
        acc = acc.wrapping_mul(10).wrapping_add(digit);
        i += 1;
    }
    if sign == -1 {
        acc.wrapping_neg()
    } else {
        acc
    }
}

// C strcmp on byte slices (no implicit NUL; compare full slices).
fn c_strcmp(a: &[u8], b: &[u8]) -> i32 {
    let n = a.len().min(b.len());
    for i in 0..n {
        let xa = a[i] as i32;
        let xb = b[i] as i32;
        if xa != xb {
            return xa - xb;
        }
    }
    (a.len() as i32) - (b.len() as i32)
}

// C strncmp on byte slices, with C behavior for n.
// Missing bytes are treated as NUL.
fn c_strncmp(a: &[u8], b: &[u8], n: usize) -> i32 {
    let mut i = 0usize;
    while i < n {
        let xa = *a.get(i).unwrap_or(&0) as i32;
        let xb = *b.get(i).unwrap_or(&0) as i32;
        if xa != xb {
            return xa - xb;
        }
        if xa == 0 {
            return 0;
        }
        i += 1;
    }
    0
}

fn c_strstr(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() {
        return true;
    }
    if needle.len() > haystack.len() {
        return false;
    }
    haystack.windows(needle.len()).any(|w| w == needle)
}

// Mimic the C parse_command:
// - split on spaces/tabs
// - command and each arg truncated to MAX_COMMAND-1 bytes
// - at most MAX_ARGS args
fn parse_command_from_line_bytes(line_bytes: &[u8]) -> (Vec<u8>, Vec<Vec<u8>>) {
    let mut buf = Vec::with_capacity(MAX_INPUT);
    buf.extend_from_slice(&line_bytes[..line_bytes.len().min(MAX_INPUT - 1)]);

    let mut parts: Vec<Vec<u8>> = Vec::new();
    let mut i = 0usize;
    while i < buf.len() {
        while i < buf.len() && (buf[i] == b' ' || buf[i] == b'\t') {
            i += 1;
        }
        if i >= buf.len() {
            break;
        }
        let start = i;
        while i < buf.len() && buf[i] != b' ' && buf[i] != b'\t' {
            i += 1;
        }
        parts.push(buf[start..i].to_vec());
    }

    if parts.is_empty() {
        return (Vec::new(), Vec::new());
    }

    let mut cmd = parts[0].clone();
    if cmd.len() >= MAX_COMMAND {
        cmd.truncate(MAX_COMMAND - 1);
    }

    let mut args: Vec<Vec<u8>> = Vec::new();
    for p in parts.into_iter().skip(1).take(MAX_ARGS) {
        let mut a = p;
        if a.len() >= MAX_COMMAND {
            a.truncate(MAX_COMMAND - 1);
        }
        args.push(a);
    }

    (cmd, args)
}

fn require_logged_in(st: &State) -> Option<usize> {
    let idx = st.current_user?;
    let u = st.users.get(idx)?;
    if u.logged_in { Some(idx) } else { None }
}

fn cmd_adduser(st: &mut State, args: &[Vec<u8>]) {
    if args.len() < 2 {
        println!("Usage: adduser <username> <password> [permission_level]");
        return;
    }
    if st.users.len() >= MAX_USERS {
        println!("Error: Maximum users reached");
        return;
    }
    let username = &args[0];
    if st.users.iter().any(|u| c_strcmp(&u.name, username) == 0) {
        println!("Error: User '{}' already exists", bytes_to_lossy_string(username));
        return;
    }
    let level = if args.len() >= 3 {
        c_atoi_wrapping(&args[2])
    } else {
        1
    };
    st.users.push(User {
        name: username.clone(),
        password: args[1].clone(),
        permission_level: level,
        logged_in: false,
    });
    println!(
        "User '{}' added with permission level {}",
        bytes_to_lossy_string(username),
        level
    );
}

fn cmd_login(st: &mut State, args: &[Vec<u8>]) {
    if args.len() < 2 {
        println!("Usage: login <username> <password>");
        return;
    }
    if let Some(idx) = st.current_user {
        if st.users.get(idx).is_some_and(|u| u.logged_in) {
            let name = bytes_to_lossy_string(&st.users[idx].name);
            println!("Error: User '{}' already logged in. Use 'logout' first.", name);
            return;
        }
    }

    let username = &args[0];
    let password = &args[1];

    if let Some(i) = st.users.iter().position(|u| c_strcmp(&u.name, username) == 0) {
        if c_strcmp(&st.users[i].password, password) == 0 {
            st.users[i].logged_in = true;
            st.current_user = Some(i);
            println!(
                "Login successful. Welcome, {}!",
                bytes_to_lossy_string(&st.users[i].name)
            );
        } else {
            println!("Error: Incorrect password");
        }
    } else {
        println!("Error: User not found");
    }
}

fn cmd_logout(st: &mut State) {
    let Some(idx) = st.current_user else {
        println!("Error: No user logged in");
        return;
    };
    if !st.users.get(idx).is_some_and(|u| u.logged_in) {
        println!("Error: No user logged in");
        return;
    }
    println!("Goodbye, {}!", bytes_to_lossy_string(&st.users[idx].name));
    st.users[idx].logged_in = false;
    st.current_user = None;
}

fn cmd_whoami(st: &State) {
    let Some(idx) = st.current_user else {
        println!("Not logged in");
        return;
    };
    let Some(u) = st.users.get(idx) else {
        println!("Not logged in");
        return;
    };
    if !u.logged_in {
        println!("Not logged in");
        return;
    }
    println!("Current user: {}", bytes_to_lossy_string(&u.name));
    println!("Permission level: {}", u.permission_level);
}

fn cmd_listusers(st: &State) {
    if st.users.is_empty() {
        println!("No users registered");
        return;
    }
    println!("Registered users:");
    for u in &st.users {
        let suffix = if u.logged_in { "[logged in]" } else { "" };
        println!(
            "  {} (level {}) {}",
            bytes_to_lossy_string(&u.name),
            u.permission_level,
            suffix
        );
    }
}

fn cmd_createfile(st: &mut State, args: &[Vec<u8>]) {
    if require_logged_in(st).is_none() {
        println!("Error: Must be logged in");
        return;
    }
    if args.is_empty() {
        println!("Usage: createfile <filename> [content]");
        return;
    }
    if st.files.len() >= MAX_FILES {
        println!("Error: Maximum files reached");
        return;
    }
    let filename = &args[0];
    if st.files.iter().any(|f| c_strcmp(&f.filename, filename) == 0) {
        println!("Error: File '{}' already exists", bytes_to_lossy_string(filename));
        return;
    }
    let owner = st.users[st.current_user.unwrap()].name.clone();
    let content = if args.len() >= 2 { args[1].clone() } else { Vec::new() };
    st.files.push(FileEntry {
        filename: filename.clone(),
        content,
        owner,
        permissions: 755,
    });
    println!("File '{}' created", bytes_to_lossy_string(filename));
}

fn cmd_readfile(st: &State, args: &[Vec<u8>]) {
    if args.is_empty() {
        println!("Usage: readfile <filename>");
        return;
    }
    let filename = &args[0];
    if let Some(f) = st.files.iter().find(|f| c_strcmp(&f.filename, filename) == 0) {
        println!("=== {} ===", bytes_to_lossy_string(&f.filename));
        println!("Owner: {}", bytes_to_lossy_string(&f.owner));
        println!("Permissions: {}", f.permissions);
        println!("Content: {}", bytes_to_lossy_string(&f.content));
    } else {
        println!("Error: File '{}' not found", bytes_to_lossy_string(filename));
    }
}

fn cmd_writefile(st: &mut State, args: &[Vec<u8>]) {
    if require_logged_in(st).is_none() {
        println!("Error: Must be logged in");
        return;
    }
    if args.len() < 2 {
        println!("Usage: writefile <filename> <content>");
        return;
    }
    let filename = &args[0];
    let content = &args[1];
    let user = &st.users[st.current_user.unwrap()];

    if let Some(f) = st.files.iter_mut().find(|f| c_strcmp(&f.filename, filename) == 0) {
        if c_strcmp(&f.owner, &user.name) == 0 || user.permission_level >= 5 {
            f.content = content.clone();
            println!("File '{}' updated", bytes_to_lossy_string(filename));
        } else {
            println!("Error: Permission denied");
        }
    } else {
        println!("Error: File '{}' not found", bytes_to_lossy_string(filename));
    }
}

fn cmd_deletefile(st: &mut State, args: &[Vec<u8>]) {
    if require_logged_in(st).is_none() {
        println!("Error: Must be logged in");
        return;
    }
    if args.is_empty() {
        println!("Usage: deletefile <filename>");
        return;
    }
    let filename = &args[0];
    let user = &st.users[st.current_user.unwrap()];

    if let Some(pos) = st.files.iter().position(|f| c_strcmp(&f.filename, filename) == 0) {
        let owner = st.files[pos].owner.clone();
        if c_strcmp(&owner, &user.name) == 0 || user.permission_level >= 9 {
            st.files.remove(pos);
            println!("File '{}' deleted", bytes_to_lossy_string(filename));
        } else {
            println!("Error: Permission denied");
        }
    } else {
        println!("Error: File '{}' not found", bytes_to_lossy_string(filename));
    }
}

fn cmd_listfiles(st: &State) {
    if st.files.is_empty() {
        println!("No files");
        return;
    }
    println!("Files:");
    for f in &st.files {
        println!(
            "  {} (owner: {}, perm: {})",
            bytes_to_lossy_string(&f.filename),
            bytes_to_lossy_string(&f.owner),
            f.permissions
        );
    }
}

fn cmd_set(st: &mut State, args: &[Vec<u8>]) {
    if args.len() < 2 {
        println!("Usage: set <name> <value>");
        return;
    }
    let name = &args[0];
    let value = &args[1];

    if let Some(v) = st.variables.iter_mut().find(|v| c_strcmp(&v.name, name) == 0) {
        v.value = value.clone();
        println!("Variable '{}' updated", bytes_to_lossy_string(name));
        return;
    }
    if st.variables.len() >= MAX_VARIABLES {
        println!("Error: Maximum variables reached");
        return;
    }
    st.variables.push(Variable {
        name: name.clone(),
        value: value.clone(),
    });
    println!("Variable '{}' set", bytes_to_lossy_string(name));
}

fn cmd_get(st: &State, args: &[Vec<u8>]) {
    if args.is_empty() {
        println!("Usage: get <name>");
        return;
    }
    let name = &args[0];
    if let Some(v) = st.variables.iter().find(|v| c_strcmp(&v.name, name) == 0) {
        println!(
            "{} = {}",
            bytes_to_lossy_string(&v.name),
            bytes_to_lossy_string(&v.value)
        );
    } else {
        println!("Error: Variable '{}' not found", bytes_to_lossy_string(name));
    }
}

fn cmd_unset(st: &mut State, args: &[Vec<u8>]) {
    if args.is_empty() {
        println!("Usage: unset <name>");
        return;
    }
    let name = &args[0];
    if let Some(pos) = st.variables.iter().position(|v| c_strcmp(&v.name, name) == 0) {
        st.variables.remove(pos);
        println!("Variable '{}' unset", bytes_to_lossy_string(name));
    } else {
        println!("Error: Variable '{}' not found", bytes_to_lossy_string(name));
    }
}

fn cmd_listvars(st: &State) {
    if st.variables.is_empty() {
        println!("No variables set");
        return;
    }
    println!("Variables:");
    for v in &st.variables {
        println!(
            "  {} = {}",
            bytes_to_lossy_string(&v.name),
            bytes_to_lossy_string(&v.value)
        );
    }
}

fn cmd_compare(args: &[Vec<u8>]) {
    if args.len() < 2 {
        println!("Usage: compare <string1> <string2>");
        return;
    }
    let a = &args[0];
    let b = &args[1];
    let result = c_strcmp(a, b);
    println!(
        "strcmp('{}', '{}') = {}",
        bytes_to_lossy_string(a),
        bytes_to_lossy_string(b),
        result
    );
    if result == 0 {
        println!("Strings are equal");
    } else if result < 0 {
        println!(
            "'{}' < '{}'",
            bytes_to_lossy_string(a),
            bytes_to_lossy_string(b)
        );
    } else {
        println!(
            "'{}' > '{}'",
            bytes_to_lossy_string(a),
            bytes_to_lossy_string(b)
        );
    }
}

fn cmd_compare_n(args: &[Vec<u8>]) {
    if args.len() < 3 {
        println!("Usage: compareN <string1> <string2> <n>");
        return;
    }
    let a = &args[0];
    let b = &args[1];
    let n_i32 = c_atoi_wrapping(&args[2]);
    let n_usize = n_i32 as usize; // C cast (negative becomes huge)
    let result = c_strncmp(a, b, n_usize);
    println!(
        "strncmp('{}', '{}', {}) = {}",
        bytes_to_lossy_string(a),
        bytes_to_lossy_string(b),
        n_i32,
        result
    );
    if result == 0 {
        println!("First {n_i32} characters are equal");
    } else if result < 0 {
        println!(
            "'{}' < '{}' (first {} chars)",
            bytes_to_lossy_string(a),
            bytes_to_lossy_string(b),
            n_i32
        );
    } else {
        println!(
            "'{}' > '{}' (first {} chars)",
            bytes_to_lossy_string(a),
            bytes_to_lossy_string(b),
            n_i32
        );
    }
}

fn cmd_startswith(args: &[Vec<u8>]) {
    if args.len() < 2 {
        println!("Usage: startswith <string> <prefix>");
        return;
    }
    let s = &args[0];
    let prefix = &args[1];
    let prefix_len = prefix.len();
    if c_strncmp(s, prefix, prefix_len) == 0 {
        println!(
            "'{}' starts with '{}'",
            bytes_to_lossy_string(s),
            bytes_to_lossy_string(prefix)
        );
    } else {
        println!(
            "'{}' does not start with '{}'",
            bytes_to_lossy_string(s),
            bytes_to_lossy_string(prefix)
        );
    }
}

fn cmd_match(args: &[Vec<u8>]) {
    if args.len() < 2 {
        println!("Usage: match <pattern> <string1> [string2] ...");
        return;
    }
    let pattern = &args[0];
    println!("Matching pattern '{}':", bytes_to_lossy_string(pattern));
    let mut matches = 0i32;
    for s in args.iter().skip(1) {
        if c_strcmp(pattern, s) == 0 {
            println!("  '{}' - EXACT MATCH", bytes_to_lossy_string(s));
            matches += 1;
        } else if c_strstr(s, pattern) {
            println!("  '{}' - contains pattern", bytes_to_lossy_string(s));
            matches += 1;
        } else {
            println!("  
// ... (truncated) ...
```

**Entity:** State

**States:** LoggedOut, LoggedIn

**Transitions:**
- LoggedOut -> LoggedIn via cmd_login()
- LoggedIn -> LoggedOut via cmd_logout()

**Evidence:** State field: `current_user: Option<usize>` encodes logged-out vs logged-in at runtime; User field: `logged_in: bool` duplicates session state and must agree with `current_user`; fn require_logged_in(st: &State) -> Option<usize>: checks `st.current_user?` then `u.logged_in`; cmd_login(): prints "Error: User '...' already logged in. Use 'logout' first." and sets `st.users[i].logged_in = true; st.current_user = Some(i);`; cmd_logout(): prints "Error: No user logged in" and sets `st.users[idx].logged_in = false; st.current_user = None;`; cmd_createfile()/cmd_writefile()/cmd_deletefile(): `if require_logged_in(st).is_none() { println!("Error: Must be logged in"); return; }` then later use `st.current_user.unwrap()`

**Implementation:** Represent session as `State<S>` with marker states `LoggedOut`/`LoggedIn`. Provide `fn login(self, ...) -> Result<State<LoggedIn>, ...>` and `fn logout(self) -> State<LoggedOut>`. In `State<LoggedIn>`, store a `CurrentUserIndex(usize)` newtype proven in-bounds (or store a `UserId`/handle), and expose auth-required commands only on `State<LoggedIn>` so `unwrap()` and repeated checks disappear.

---

## Precondition Invariants

### 5. Authorization protocol for file mutation (NoAuth / AuthorizedAsOwner / AuthorizedAsAdmin)

**Location**: `/data/test_case/main.rs:1-584`

**Confidence**: high

**Suggested Pattern**: capability

**Description**: File mutation commands implicitly require both (1) a logged-in user and (2) an authorization check: owner match or sufficient `permission_level`. These are enforced with runtime branching and string comparisons of owner/name bytes. The type system does not distinguish authenticated-and-authorized access from unauthenticated access, so callers can reach mutation paths only guarded by runtime checks and error messages.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct User; struct FileEntry; struct Variable; struct State, 19 free function(s)

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![feature(c_variadic)]
#![feature(extern_types)]
#![feature(linkage)]
#![feature(rustc_private)]
#![feature(thread_local)]
#![feature(formatting_options)]
#![feature(as_array_of_cells)]

use std::io::{self, Write};

const MAX_INPUT: usize = 256;
const MAX_COMMAND: usize = 64;
const MAX_ARGS: usize = 10;
const MAX_FILES: usize = 20;
const MAX_USERS: usize = 10;
const MAX_VARIABLES: usize = 20;

#[derive(Clone, Default)]
struct User {
    name: Vec<u8>,
    password: Vec<u8>,
    permission_level: i32,
    logged_in: bool,
}

#[derive(Clone, Default)]
struct FileEntry {
    filename: Vec<u8>,
    content: Vec<u8>,
    owner: Vec<u8>,
    permissions: i32,
}

#[derive(Clone, Default)]
struct Variable {
    name: Vec<u8>,
    value: Vec<u8>,
}

#[derive(Default)]
struct State {
    users: Vec<User>,
    current_user: Option<usize>,
    files: Vec<FileEntry>,
    variables: Vec<Variable>,
    debug_mode: bool,
    verbose_mode: bool,
}

fn bytes_to_lossy_string(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

// C-like atoi: parses optional whitespace, optional sign, then digits; stops at first non-digit.
// Uses wrapping arithmetic to mimic typical C overflow behavior.
fn c_atoi_wrapping(s: &[u8]) -> i32 {
    let mut i = 0usize;
    while i < s.len() && matches!(s[i], b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c) {
        i += 1;
    }
    let mut sign: i32 = 1;
    if i < s.len() {
        if s[i] == b'-' {
            sign = -1;
            i += 1;
        } else if s[i] == b'+' {
            i += 1;
        }
    }
    let mut acc: i32 = 0;
    while i < s.len() {
        let c = s[i];
        if !(b'0'..=b'9').contains(&c) {
            break;
        }
        let digit = (c - b'0') as i32;
        acc = acc.wrapping_mul(10).wrapping_add(digit);
        i += 1;
    }
    if sign == -1 {
        acc.wrapping_neg()
    } else {
        acc
    }
}

// C strcmp on byte slices (no implicit NUL; compare full slices).
fn c_strcmp(a: &[u8], b: &[u8]) -> i32 {
    let n = a.len().min(b.len());
    for i in 0..n {
        let xa = a[i] as i32;
        let xb = b[i] as i32;
        if xa != xb {
            return xa - xb;
        }
    }
    (a.len() as i32) - (b.len() as i32)
}

// C strncmp on byte slices, with C behavior for n.
// Missing bytes are treated as NUL.
fn c_strncmp(a: &[u8], b: &[u8], n: usize) -> i32 {
    let mut i = 0usize;
    while i < n {
        let xa = *a.get(i).unwrap_or(&0) as i32;
        let xb = *b.get(i).unwrap_or(&0) as i32;
        if xa != xb {
            return xa - xb;
        }
        if xa == 0 {
            return 0;
        }
        i += 1;
    }
    0
}

fn c_strstr(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() {
        return true;
    }
    if needle.len() > haystack.len() {
        return false;
    }
    haystack.windows(needle.len()).any(|w| w == needle)
}

// Mimic the C parse_command:
// - split on spaces/tabs
// - command and each arg truncated to MAX_COMMAND-1 bytes
// - at most MAX_ARGS args
fn parse_command_from_line_bytes(line_bytes: &[u8]) -> (Vec<u8>, Vec<Vec<u8>>) {
    let mut buf = Vec::with_capacity(MAX_INPUT);
    buf.extend_from_slice(&line_bytes[..line_bytes.len().min(MAX_INPUT - 1)]);

    let mut parts: Vec<Vec<u8>> = Vec::new();
    let mut i = 0usize;
    while i < buf.len() {
        while i < buf.len() && (buf[i] == b' ' || buf[i] == b'\t') {
            i += 1;
        }
        if i >= buf.len() {
            break;
        }
        let start = i;
        while i < buf.len() && buf[i] != b' ' && buf[i] != b'\t' {
            i += 1;
        }
        parts.push(buf[start..i].to_vec());
    }

    if parts.is_empty() {
        return (Vec::new(), Vec::new());
    }

    let mut cmd = parts[0].clone();
    if cmd.len() >= MAX_COMMAND {
        cmd.truncate(MAX_COMMAND - 1);
    }

    let mut args: Vec<Vec<u8>> = Vec::new();
    for p in parts.into_iter().skip(1).take(MAX_ARGS) {
        let mut a = p;
        if a.len() >= MAX_COMMAND {
            a.truncate(MAX_COMMAND - 1);
        }
        args.push(a);
    }

    (cmd, args)
}

fn require_logged_in(st: &State) -> Option<usize> {
    let idx = st.current_user?;
    let u = st.users.get(idx)?;
    if u.logged_in { Some(idx) } else { None }
}

fn cmd_adduser(st: &mut State, args: &[Vec<u8>]) {
    if args.len() < 2 {
        println!("Usage: adduser <username> <password> [permission_level]");
        return;
    }
    if st.users.len() >= MAX_USERS {
        println!("Error: Maximum users reached");
        return;
    }
    let username = &args[0];
    if st.users.iter().any(|u| c_strcmp(&u.name, username) == 0) {
        println!("Error: User '{}' already exists", bytes_to_lossy_string(username));
        return;
    }
    let level = if args.len() >= 3 {
        c_atoi_wrapping(&args[2])
    } else {
        1
    };
    st.users.push(User {
        name: username.clone(),
        password: args[1].clone(),
        permission_level: level,
        logged_in: false,
    });
    println!(
        "User '{}' added with permission level {}",
        bytes_to_lossy_string(username),
        level
    );
}

fn cmd_login(st: &mut State, args: &[Vec<u8>]) {
    if args.len() < 2 {
        println!("Usage: login <username> <password>");
        return;
    }
    if let Some(idx) = st.current_user {
        if st.users.get(idx).is_some_and(|u| u.logged_in) {
            let name = bytes_to_lossy_string(&st.users[idx].name);
            println!("Error: User '{}' already logged in. Use 'logout' first.", name);
            return;
        }
    }

    let username = &args[0];
    let password = &args[1];

    if let Some(i) = st.users.iter().position(|u| c_strcmp(&u.name, username) == 0) {
        if c_strcmp(&st.users[i].password, password) == 0 {
            st.users[i].logged_in = true;
            st.current_user = Some(i);
            println!(
                "Login successful. Welcome, {}!",
                bytes_to_lossy_string(&st.users[i].name)
            );
        } else {
            println!("Error: Incorrect password");
        }
    } else {
        println!("Error: User not found");
    }
}

fn cmd_logout(st: &mut State) {
    let Some(idx) = st.current_user else {
        println!("Error: No user logged in");
        return;
    };
    if !st.users.get(idx).is_some_and(|u| u.logged_in) {
        println!("Error: No user logged in");
        return;
    }
    println!("Goodbye, {}!", bytes_to_lossy_string(&st.users[idx].name));
    st.users[idx].logged_in = false;
    st.current_user = None;
}

fn cmd_whoami(st: &State) {
    let Some(idx) = st.current_user else {
        println!("Not logged in");
        return;
    };
    let Some(u) = st.users.get(idx) else {
        println!("Not logged in");
        return;
    };
    if !u.logged_in {
        println!("Not logged in");
        return;
    }
    println!("Current user: {}", bytes_to_lossy_string(&u.name));
    println!("Permission level: {}", u.permission_level);
}

fn cmd_listusers(st: &State) {
    if st.users.is_empty() {
        println!("No users registered");
        return;
    }
    println!("Registered users:");
    for u in &st.users {
        let suffix = if u.logged_in { "[logged in]" } else { "" };
        println!(
            "  {} (level {}) {}",
            bytes_to_lossy_string(&u.name),
            u.permission_level,
            suffix
        );
    }
}

fn cmd_createfile(st: &mut State, args: &[Vec<u8>]) {
    if require_logged_in(st).is_none() {
        println!("Error: Must be logged in");
        return;
    }
    if args.is_empty() {
        println!("Usage: createfile <filename> [content]");
        return;
    }
    if st.files.len() >= MAX_FILES {
        println!("Error: Maximum files reached");
        return;
    }
    let filename = &args[0];
    if st.files.iter().any(|f| c_strcmp(&f.filename, filename) == 0) {
        println!("Error: File '{}' already exists", bytes_to_lossy_string(filename));
        return;
    }
    let owner = st.users[st.current_user.unwrap()].name.clone();
    let content = if args.len() >= 2 { args[1].clone() } else { Vec::new() };
    st.files.push(FileEntry {
        filename: filename.clone(),
        content,
        owner,
        permissions: 755,
    });
    println!("File '{}' created", bytes_to_lossy_string(filename));
}

fn cmd_readfile(st: &State, args: &[Vec<u8>]) {
    if args.is_empty() {
        println!("Usage: readfile <filename>");
        return;
    }
    let filename = &args[0];
    if let Some(f) = st.files.iter().find(|f| c_strcmp(&f.filename, filename) == 0) {
        println!("=== {} ===", bytes_to_lossy_string(&f.filename));
        println!("Owner: {}", bytes_to_lossy_string(&f.owner));
        println!("Permissions: {}", f.permissions);
        println!("Content: {}", bytes_to_lossy_string(&f.content));
    } else {
        println!("Error: File '{}' not found", bytes_to_lossy_string(filename));
    }
}

fn cmd_writefile(st: &mut State, args: &[Vec<u8>]) {
    if require_logged_in(st).is_none() {
        println!("Error: Must be logged in");
        return;
    }
    if args.len() < 2 {
        println!("Usage: writefile <filename> <content>");
        return;
    }
    let filename = &args[0];
    let content = &args[1];
    let user = &st.users[st.current_user.unwrap()];

    if let Some(f) = st.files.iter_mut().find(|f| c_strcmp(&f.filename, filename) == 0) {
        if c_strcmp(&f.owner, &user.name) == 0 || user.permission_level >= 5 {
            f.content = content.clone();
            println!("File '{}' updated", bytes_to_lossy_string(filename));
        } else {
            println!("Error: Permission denied");
        }
    } else {
        println!("Error: File '{}' not found", bytes_to_lossy_string(filename));
    }
}

fn cmd_deletefile(st: &mut State, args: &[Vec<u8>]) {
    if require_logged_in(st).is_none() {
        println!("Error: Must be logged in");
        return;
    }
    if args.is_empty() {
        println!("Usage: deletefile <filename>");
        return;
    }
    let filename = &args[0];
    let user = &st.users[st.current_user.unwrap()];

    if let Some(pos) = st.files.iter().position(|f| c_strcmp(&f.filename, filename) == 0) {
        let owner = st.files[pos].owner.clone();
        if c_strcmp(&owner, &user.name) == 0 || user.permission_level >= 9 {
            st.files.remove(pos);
            println!("File '{}' deleted", bytes_to_lossy_string(filename));
        } else {
            println!("Error: Permission denied");
        }
    } else {
        println!("Error: File '{}' not found", bytes_to_lossy_string(filename));
    }
}

fn cmd_listfiles(st: &State) {
    if st.files.is_empty() {
        println!("No files");
        return;
    }
    println!("Files:");
    for f in &st.files {
        println!(
            "  {} (owner: {}, perm: {})",
            bytes_to_lossy_string(&f.filename),
            bytes_to_lossy_string(&f.owner),
            f.permissions
        );
    }
}

fn cmd_set(st: &mut State, args: &[Vec<u8>]) {
    if args.len() < 2 {
        println!("Usage: set <name> <value>");
        return;
    }
    let name = &args[0];
    let value = &args[1];

    if let Some(v) = st.variables.iter_mut().find(|v| c_strcmp(&v.name, name) == 0) {
        v.value = value.clone();
        println!("Variable '{}' updated", bytes_to_lossy_string(name));
        return;
    }
    if st.variables.len() >= MAX_VARIABLES {
        println!("Error: Maximum variables reached");
        return;
    }
    st.variables.push(Variable {
        name: name.clone(),
        value: value.clone(),
    });
    println!("Variable '{}' set", bytes_to_lossy_string(name));
}

fn cmd_get(st: &State, args: &[Vec<u8>]) {
    if args.is_empty() {
        println!("Usage: get <name>");
        return;
    }
    let name = &args[0];
    if let Some(v) = st.variables.iter().find(|v| c_strcmp(&v.name, name) == 0) {
        println!(
            "{} = {}",
            bytes_to_lossy_string(&v.name),
            bytes_to_lossy_string(&v.value)
        );
    } else {
        println!("Error: Variable '{}' not found", bytes_to_lossy_string(name));
    }
}

fn cmd_unset(st: &mut State, args: &[Vec<u8>]) {
    if args.is_empty() {
        println!("Usage: unset <name>");
        return;
    }
    let name = &args[0];
    if let Some(pos) = st.variables.iter().position(|v| c_strcmp(&v.name, name) == 0) {
        st.variables.remove(pos);
        println!("Variable '{}' unset", bytes_to_lossy_string(name));
    } else {
        println!("Error: Variable '{}' not found", bytes_to_lossy_string(name));
    }
}

fn cmd_listvars(st: &State) {
    if st.variables.is_empty() {
        println!("No variables set");
        return;
    }
    println!("Variables:");
    for v in &st.variables {
        println!(
            "  {} = {}",
            bytes_to_lossy_string(&v.name),
            bytes_to_lossy_string(&v.value)
        );
    }
}

fn cmd_compare(args: &[Vec<u8>]) {
    if args.len() < 2 {
        println!("Usage: compare <string1> <string2>");
        return;
    }
    let a = &args[0];
    let b = &args[1];
    let result = c_strcmp(a, b);
    println!(
        "strcmp('{}', '{}') = {}",
        bytes_to_lossy_string(a),
        bytes_to_lossy_string(b),
        result
    );
    if result == 0 {
        println!("Strings are equal");
    } else if result < 0 {
        println!(
            "'{}' < '{}'",
            bytes_to_lossy_string(a),
            bytes_to_lossy_string(b)
        );
    } else {
        println!(
            "'{}' > '{}'",
            bytes_to_lossy_string(a),
            bytes_to_lossy_string(b)
        );
    }
}

fn cmd_compare_n(args: &[Vec<u8>]) {
    if args.len() < 3 {
        println!("Usage: compareN <string1> <string2> <n>");
        return;
    }
    let a = &args[0];
    let b = &args[1];
    let n_i32 = c_atoi_wrapping(&args[2]);
    let n_usize = n_i32 as usize; // C cast (negative becomes huge)
    let result = c_strncmp(a, b, n_usize);
    println!(
        "strncmp('{}', '{}', {}) = {}",
        bytes_to_lossy_string(a),
        bytes_to_lossy_string(b),
        n_i32,
        result
    );
    if result == 0 {
        println!("First {n_i32} characters are equal");
    } else if result < 0 {
        println!(
            "'{}' < '{}' (first {} chars)",
            bytes_to_lossy_string(a),
            bytes_to_lossy_string(b),
            n_i32
        );
    } else {
        println!(
            "'{}' > '{}' (first {} chars)",
            bytes_to_lossy_string(a),
            bytes_to_lossy_string(b),
            n_i32
        );
    }
}

fn cmd_startswith(args: &[Vec<u8>]) {
    if args.len() < 2 {
        println!("Usage: startswith <string> <prefix>");
        return;
    }
    let s = &args[0];
    let prefix = &args[1];
    let prefix_len = prefix.len();
    if c_strncmp(s, prefix, prefix_len) == 0 {
        println!(
            "'{}' starts with '{}'",
            bytes_to_lossy_string(s),
            bytes_to_lossy_string(prefix)
        );
    } else {
        println!(
            "'{}' does not start with '{}'",
            bytes_to_lossy_string(s),
            bytes_to_lossy_string(prefix)
        );
    }
}

fn cmd_match(args: &[Vec<u8>]) {
    if args.len() < 2 {
        println!("Usage: match <pattern> <string1> [string2] ...");
        return;
    }
    let pattern = &args[0];
    println!("Matching pattern '{}':", bytes_to_lossy_string(pattern));
    let mut matches = 0i32;
    for s in args.iter().skip(1) {
        if c_strcmp(pattern, s) == 0 {
            println!("  '{}' - EXACT MATCH", bytes_to_lossy_string(s));
            matches += 1;
        } else if c_strstr(s, pattern) {
            println!("  '{}' - contains pattern", bytes_to_lossy_string(s));
            matches += 1;
        } else {
            println!("  
// ... (truncated) ...
```

**Entity:** State / FileEntry

**States:** NoAuth, AuthorizedAsOwner, AuthorizedAsAdmin

**Transitions:**
- NoAuth -> AuthorizedAsOwner via successful login + choosing a file where `f.owner == user.name`
- NoAuth -> AuthorizedAsAdmin via successful login + `user.permission_level >= 5` (write) or `>= 9` (delete)

**Evidence:** cmd_writefile(): requires login (`"Error: Must be logged in"`), then authorizes with `c_strcmp(&f.owner, &user.name) == 0 || user.permission_level >= 5`, else prints "Error: Permission denied"; cmd_deletefile(): requires login, then authorizes with `c_strcmp(&owner, &user.name) == 0 || user.permission_level >= 9`, else prints "Error: Permission denied"; FileEntry fields: `owner: Vec<u8>`, `permissions: i32` are used for access decisions but are not typed as identities/ACLs; cmd_createfile(): after login, uses `st.current_user.unwrap()` to fetch owner name for new file

**Implementation:** Introduce capabilities like `struct Authenticated<'a> { st: &'a mut State, user: UserRef }` and `struct CanWriteFile` / `CanDeleteFile` tokens produced by checked constructors (`authorize_write(file, user) -> Option<CanWriteFile>`). Expose `write_content(&mut self, cap: CanWriteFile, ...)` so mutation is only possible when authorization has been proven.

---

## Protocol Invariants

### 4. User login flag consistency with global session (Inactive / ActiveCurrent)

**Location**: `/data/test_case/main.rs:1-584`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: A user's `logged_in: bool` is used as part of the authentication model, but the code effectively assumes at most one active session and that the logged-in user corresponds to `State.current_user`. This is enforced by runtime checks and assignments in cmd_login/cmd_logout, not by types; nothing prevents stale `logged_in=true` on a non-current user or multiple `logged_in=true` users if logic changes.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct User; struct FileEntry; struct Variable; struct State, 19 free function(s)

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![feature(c_variadic)]
#![feature(extern_types)]
#![feature(linkage)]
#![feature(rustc_private)]
#![feature(thread_local)]
#![feature(formatting_options)]
#![feature(as_array_of_cells)]

use std::io::{self, Write};

const MAX_INPUT: usize = 256;
const MAX_COMMAND: usize = 64;
const MAX_ARGS: usize = 10;
const MAX_FILES: usize = 20;
const MAX_USERS: usize = 10;
const MAX_VARIABLES: usize = 20;

#[derive(Clone, Default)]
struct User {
    name: Vec<u8>,
    password: Vec<u8>,
    permission_level: i32,
    logged_in: bool,
}

#[derive(Clone, Default)]
struct FileEntry {
    filename: Vec<u8>,
    content: Vec<u8>,
    owner: Vec<u8>,
    permissions: i32,
}

#[derive(Clone, Default)]
struct Variable {
    name: Vec<u8>,
    value: Vec<u8>,
}

#[derive(Default)]
struct State {
    users: Vec<User>,
    current_user: Option<usize>,
    files: Vec<FileEntry>,
    variables: Vec<Variable>,
    debug_mode: bool,
    verbose_mode: bool,
}

fn bytes_to_lossy_string(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

// C-like atoi: parses optional whitespace, optional sign, then digits; stops at first non-digit.
// Uses wrapping arithmetic to mimic typical C overflow behavior.
fn c_atoi_wrapping(s: &[u8]) -> i32 {
    let mut i = 0usize;
    while i < s.len() && matches!(s[i], b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c) {
        i += 1;
    }
    let mut sign: i32 = 1;
    if i < s.len() {
        if s[i] == b'-' {
            sign = -1;
            i += 1;
        } else if s[i] == b'+' {
            i += 1;
        }
    }
    let mut acc: i32 = 0;
    while i < s.len() {
        let c = s[i];
        if !(b'0'..=b'9').contains(&c) {
            break;
        }
        let digit = (c - b'0') as i32;
        acc = acc.wrapping_mul(10).wrapping_add(digit);
        i += 1;
    }
    if sign == -1 {
        acc.wrapping_neg()
    } else {
        acc
    }
}

// C strcmp on byte slices (no implicit NUL; compare full slices).
fn c_strcmp(a: &[u8], b: &[u8]) -> i32 {
    let n = a.len().min(b.len());
    for i in 0..n {
        let xa = a[i] as i32;
        let xb = b[i] as i32;
        if xa != xb {
            return xa - xb;
        }
    }
    (a.len() as i32) - (b.len() as i32)
}

// C strncmp on byte slices, with C behavior for n.
// Missing bytes are treated as NUL.
fn c_strncmp(a: &[u8], b: &[u8], n: usize) -> i32 {
    let mut i = 0usize;
    while i < n {
        let xa = *a.get(i).unwrap_or(&0) as i32;
        let xb = *b.get(i).unwrap_or(&0) as i32;
        if xa != xb {
            return xa - xb;
        }
        if xa == 0 {
            return 0;
        }
        i += 1;
    }
    0
}

fn c_strstr(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() {
        return true;
    }
    if needle.len() > haystack.len() {
        return false;
    }
    haystack.windows(needle.len()).any(|w| w == needle)
}

// Mimic the C parse_command:
// - split on spaces/tabs
// - command and each arg truncated to MAX_COMMAND-1 bytes
// - at most MAX_ARGS args
fn parse_command_from_line_bytes(line_bytes: &[u8]) -> (Vec<u8>, Vec<Vec<u8>>) {
    let mut buf = Vec::with_capacity(MAX_INPUT);
    buf.extend_from_slice(&line_bytes[..line_bytes.len().min(MAX_INPUT - 1)]);

    let mut parts: Vec<Vec<u8>> = Vec::new();
    let mut i = 0usize;
    while i < buf.len() {
        while i < buf.len() && (buf[i] == b' ' || buf[i] == b'\t') {
            i += 1;
        }
        if i >= buf.len() {
            break;
        }
        let start = i;
        while i < buf.len() && buf[i] != b' ' && buf[i] != b'\t' {
            i += 1;
        }
        parts.push(buf[start..i].to_vec());
    }

    if parts.is_empty() {
        return (Vec::new(), Vec::new());
    }

    let mut cmd = parts[0].clone();
    if cmd.len() >= MAX_COMMAND {
        cmd.truncate(MAX_COMMAND - 1);
    }

    let mut args: Vec<Vec<u8>> = Vec::new();
    for p in parts.into_iter().skip(1).take(MAX_ARGS) {
        let mut a = p;
        if a.len() >= MAX_COMMAND {
            a.truncate(MAX_COMMAND - 1);
        }
        args.push(a);
    }

    (cmd, args)
}

fn require_logged_in(st: &State) -> Option<usize> {
    let idx = st.current_user?;
    let u = st.users.get(idx)?;
    if u.logged_in { Some(idx) } else { None }
}

fn cmd_adduser(st: &mut State, args: &[Vec<u8>]) {
    if args.len() < 2 {
        println!("Usage: adduser <username> <password> [permission_level]");
        return;
    }
    if st.users.len() >= MAX_USERS {
        println!("Error: Maximum users reached");
        return;
    }
    let username = &args[0];
    if st.users.iter().any(|u| c_strcmp(&u.name, username) == 0) {
        println!("Error: User '{}' already exists", bytes_to_lossy_string(username));
        return;
    }
    let level = if args.len() >= 3 {
        c_atoi_wrapping(&args[2])
    } else {
        1
    };
    st.users.push(User {
        name: username.clone(),
        password: args[1].clone(),
        permission_level: level,
        logged_in: false,
    });
    println!(
        "User '{}' added with permission level {}",
        bytes_to_lossy_string(username),
        level
    );
}

fn cmd_login(st: &mut State, args: &[Vec<u8>]) {
    if args.len() < 2 {
        println!("Usage: login <username> <password>");
        return;
    }
    if let Some(idx) = st.current_user {
        if st.users.get(idx).is_some_and(|u| u.logged_in) {
            let name = bytes_to_lossy_string(&st.users[idx].name);
            println!("Error: User '{}' already logged in. Use 'logout' first.", name);
            return;
        }
    }

    let username = &args[0];
    let password = &args[1];

    if let Some(i) = st.users.iter().position(|u| c_strcmp(&u.name, username) == 0) {
        if c_strcmp(&st.users[i].password, password) == 0 {
            st.users[i].logged_in = true;
            st.current_user = Some(i);
            println!(
                "Login successful. Welcome, {}!",
                bytes_to_lossy_string(&st.users[i].name)
            );
        } else {
            println!("Error: Incorrect password");
        }
    } else {
        println!("Error: User not found");
    }
}

fn cmd_logout(st: &mut State) {
    let Some(idx) = st.current_user else {
        println!("Error: No user logged in");
        return;
    };
    if !st.users.get(idx).is_some_and(|u| u.logged_in) {
        println!("Error: No user logged in");
        return;
    }
    println!("Goodbye, {}!", bytes_to_lossy_string(&st.users[idx].name));
    st.users[idx].logged_in = false;
    st.current_user = None;
}

fn cmd_whoami(st: &State) {
    let Some(idx) = st.current_user else {
        println!("Not logged in");
        return;
    };
    let Some(u) = st.users.get(idx) else {
        println!("Not logged in");
        return;
    };
    if !u.logged_in {
        println!("Not logged in");
        return;
    }
    println!("Current user: {}", bytes_to_lossy_string(&u.name));
    println!("Permission level: {}", u.permission_level);
}

fn cmd_listusers(st: &State) {
    if st.users.is_empty() {
        println!("No users registered");
        return;
    }
    println!("Registered users:");
    for u in &st.users {
        let suffix = if u.logged_in { "[logged in]" } else { "" };
        println!(
            "  {} (level {}) {}",
            bytes_to_lossy_string(&u.name),
            u.permission_level,
            suffix
        );
    }
}

fn cmd_createfile(st: &mut State, args: &[Vec<u8>]) {
    if require_logged_in(st).is_none() {
        println!("Error: Must be logged in");
        return;
    }
    if args.is_empty() {
        println!("Usage: createfile <filename> [content]");
        return;
    }
    if st.files.len() >= MAX_FILES {
        println!("Error: Maximum files reached");
        return;
    }
    let filename = &args[0];
    if st.files.iter().any(|f| c_strcmp(&f.filename, filename) == 0) {
        println!("Error: File '{}' already exists", bytes_to_lossy_string(filename));
        return;
    }
    let owner = st.users[st.current_user.unwrap()].name.clone();
    let content = if args.len() >= 2 { args[1].clone() } else { Vec::new() };
    st.files.push(FileEntry {
        filename: filename.clone(),
        content,
        owner,
        permissions: 755,
    });
    println!("File '{}' created", bytes_to_lossy_string(filename));
}

fn cmd_readfile(st: &State, args: &[Vec<u8>]) {
    if args.is_empty() {
        println!("Usage: readfile <filename>");
        return;
    }
    let filename = &args[0];
    if let Some(f) = st.files.iter().find(|f| c_strcmp(&f.filename, filename) == 0) {
        println!("=== {} ===", bytes_to_lossy_string(&f.filename));
        println!("Owner: {}", bytes_to_lossy_string(&f.owner));
        println!("Permissions: {}", f.permissions);
        println!("Content: {}", bytes_to_lossy_string(&f.content));
    } else {
        println!("Error: File '{}' not found", bytes_to_lossy_string(filename));
    }
}

fn cmd_writefile(st: &mut State, args: &[Vec<u8>]) {
    if require_logged_in(st).is_none() {
        println!("Error: Must be logged in");
        return;
    }
    if args.len() < 2 {
        println!("Usage: writefile <filename> <content>");
        return;
    }
    let filename = &args[0];
    let content = &args[1];
    let user = &st.users[st.current_user.unwrap()];

    if let Some(f) = st.files.iter_mut().find(|f| c_strcmp(&f.filename, filename) == 0) {
        if c_strcmp(&f.owner, &user.name) == 0 || user.permission_level >= 5 {
            f.content = content.clone();
            println!("File '{}' updated", bytes_to_lossy_string(filename));
        } else {
            println!("Error: Permission denied");
        }
    } else {
        println!("Error: File '{}' not found", bytes_to_lossy_string(filename));
    }
}

fn cmd_deletefile(st: &mut State, args: &[Vec<u8>]) {
    if require_logged_in(st).is_none() {
        println!("Error: Must be logged in");
        return;
    }
    if args.is_empty() {
        println!("Usage: deletefile <filename>");
        return;
    }
    let filename = &args[0];
    let user = &st.users[st.current_user.unwrap()];

    if let Some(pos) = st.files.iter().position(|f| c_strcmp(&f.filename, filename) == 0) {
        let owner = st.files[pos].owner.clone();
        if c_strcmp(&owner, &user.name) == 0 || user.permission_level >= 9 {
            st.files.remove(pos);
            println!("File '{}' deleted", bytes_to_lossy_string(filename));
        } else {
            println!("Error: Permission denied");
        }
    } else {
        println!("Error: File '{}' not found", bytes_to_lossy_string(filename));
    }
}

fn cmd_listfiles(st: &State) {
    if st.files.is_empty() {
        println!("No files");
        return;
    }
    println!("Files:");
    for f in &st.files {
        println!(
            "  {} (owner: {}, perm: {})",
            bytes_to_lossy_string(&f.filename),
            bytes_to_lossy_string(&f.owner),
            f.permissions
        );
    }
}

fn cmd_set(st: &mut State, args: &[Vec<u8>]) {
    if args.len() < 2 {
        println!("Usage: set <name> <value>");
        return;
    }
    let name = &args[0];
    let value = &args[1];

    if let Some(v) = st.variables.iter_mut().find(|v| c_strcmp(&v.name, name) == 0) {
        v.value = value.clone();
        println!("Variable '{}' updated", bytes_to_lossy_string(name));
        return;
    }
    if st.variables.len() >= MAX_VARIABLES {
        println!("Error: Maximum variables reached");
        return;
    }
    st.variables.push(Variable {
        name: name.clone(),
        value: value.clone(),
    });
    println!("Variable '{}' set", bytes_to_lossy_string(name));
}

fn cmd_get(st: &State, args: &[Vec<u8>]) {
    if args.is_empty() {
        println!("Usage: get <name>");
        return;
    }
    let name = &args[0];
    if let Some(v) = st.variables.iter().find(|v| c_strcmp(&v.name, name) == 0) {
        println!(
            "{} = {}",
            bytes_to_lossy_string(&v.name),
            bytes_to_lossy_string(&v.value)
        );
    } else {
        println!("Error: Variable '{}' not found", bytes_to_lossy_string(name));
    }
}

fn cmd_unset(st: &mut State, args: &[Vec<u8>]) {
    if args.is_empty() {
        println!("Usage: unset <name>");
        return;
    }
    let name = &args[0];
    if let Some(pos) = st.variables.iter().position(|v| c_strcmp(&v.name, name) == 0) {
        st.variables.remove(pos);
        println!("Variable '{}' unset", bytes_to_lossy_string(name));
    } else {
        println!("Error: Variable '{}' not found", bytes_to_lossy_string(name));
    }
}

fn cmd_listvars(st: &State) {
    if st.variables.is_empty() {
        println!("No variables set");
        return;
    }
    println!("Variables:");
    for v in &st.variables {
        println!(
            "  {} = {}",
            bytes_to_lossy_string(&v.name),
            bytes_to_lossy_string(&v.value)
        );
    }
}

fn cmd_compare(args: &[Vec<u8>]) {
    if args.len() < 2 {
        println!("Usage: compare <string1> <string2>");
        return;
    }
    let a = &args[0];
    let b = &args[1];
    let result = c_strcmp(a, b);
    println!(
        "strcmp('{}', '{}') = {}",
        bytes_to_lossy_string(a),
        bytes_to_lossy_string(b),
        result
    );
    if result == 0 {
        println!("Strings are equal");
    } else if result < 0 {
        println!(
            "'{}' < '{}'",
            bytes_to_lossy_string(a),
            bytes_to_lossy_string(b)
        );
    } else {
        println!(
            "'{}' > '{}'",
            bytes_to_lossy_string(a),
            bytes_to_lossy_string(b)
        );
    }
}

fn cmd_compare_n(args: &[Vec<u8>]) {
    if args.len() < 3 {
        println!("Usage: compareN <string1> <string2> <n>");
        return;
    }
    let a = &args[0];
    let b = &args[1];
    let n_i32 = c_atoi_wrapping(&args[2]);
    let n_usize = n_i32 as usize; // C cast (negative becomes huge)
    let result = c_strncmp(a, b, n_usize);
    println!(
        "strncmp('{}', '{}', {}) = {}",
        bytes_to_lossy_string(a),
        bytes_to_lossy_string(b),
        n_i32,
        result
    );
    if result == 0 {
        println!("First {n_i32} characters are equal");
    } else if result < 0 {
        println!(
            "'{}' < '{}' (first {} chars)",
            bytes_to_lossy_string(a),
            bytes_to_lossy_string(b),
            n_i32
        );
    } else {
        println!(
            "'{}' > '{}' (first {} chars)",
            bytes_to_lossy_string(a),
            bytes_to_lossy_string(b),
            n_i32
        );
    }
}

fn cmd_startswith(args: &[Vec<u8>]) {
    if args.len() < 2 {
        println!("Usage: startswith <string> <prefix>");
        return;
    }
    let s = &args[0];
    let prefix = &args[1];
    let prefix_len = prefix.len();
    if c_strncmp(s, prefix, prefix_len) == 0 {
        println!(
            "'{}' starts with '{}'",
            bytes_to_lossy_string(s),
            bytes_to_lossy_string(prefix)
        );
    } else {
        println!(
            "'{}' does not start with '{}'",
            bytes_to_lossy_string(s),
            bytes_to_lossy_string(prefix)
        );
    }
}

fn cmd_match(args: &[Vec<u8>]) {
    if args.len() < 2 {
        println!("Usage: match <pattern> <string1> [string2] ...");
        return;
    }
    let pattern = &args[0];
    println!("Matching pattern '{}':", bytes_to_lossy_string(pattern));
    let mut matches = 0i32;
    for s in args.iter().skip(1) {
        if c_strcmp(pattern, s) == 0 {
            println!("  '{}' - EXACT MATCH", bytes_to_lossy_string(s));
            matches += 1;
        } else if c_strstr(s, pattern) {
            println!("  '{}' - contains pattern", bytes_to_lossy_string(s));
            matches += 1;
        } else {
            println!("  
// ... (truncated) ...
```

**Entity:** User

**States:** Inactive, ActiveCurrent

**Transitions:**
- Inactive -> ActiveCurrent via cmd_login() setting `logged_in = true` and `State.current_user = Some(i)`
- ActiveCurrent -> Inactive via cmd_logout() setting `logged_in = false` and `State.current_user = None`

**Evidence:** User field: `logged_in: bool` is a per-user state flag; cmd_login(): guards on `st.current_user` + `u.logged_in` and then sets `st.users[i].logged_in = true`; cmd_logout(): checks `st.current_user` and `u.logged_in`, then sets `st.users[idx].logged_in = false`; cmd_listusers(): prints suffix `"[logged in]"` based on `u.logged_in`, implying meaningful per-user login state

**Implementation:** Eliminate `User.logged_in` and instead model the active session as a capability/token: `struct Session { user_index: usize }` created only by `login`. Commands requiring authentication take `(&mut State, &Session)` (or are methods on `Session`) so "who is logged in" is a single source of truth and cannot drift.

---


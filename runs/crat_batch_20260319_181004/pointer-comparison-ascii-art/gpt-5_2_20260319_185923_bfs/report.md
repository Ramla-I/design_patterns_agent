# Latent Invariant Analysis Report

## Summary

- **Total invariants discovered**: 4
- **Temporal ordering**: 0
- **Resource lifecycle**: 1
- **State machine**: 1
- **Precondition**: 1
- **Protocol**: 1
- **Modules analyzed**: 1

## Resource Lifecycle Invariants

### 2. Global Shape registry lifecycle (Uninitialized/Initialized/Cleaned-up) + valid ShapeId range

**Location**: `/data/test_case/main.rs:1-652`

**Confidence**: high

**Suggested Pattern**: typestate

**Description**: The code relies on a thread-local global registry of singleton Shapes. Callers are expected to initialize it (shape_manager_init) before using shape_get/view_all_shapes/scene_load_internal; cleanup clears it. The API exposes raw pointers (*const Shape) and uses null as an error sentinel when the registry is empty or when an out-of-range shape_type is requested. None of these requirements are expressed in types: callers can obtain null pointers and later dereference them (some places check, others convert to Option via as_ref/as_mut). The valid shape identifier domain is also implicit: type_ must be < SHAPE_COUNT.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Shape, 4 free function(s); struct Scene, 7 free function(s)

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

use std::cell::{Cell, RefCell};
use std::io::{self, BufRead, Write};

type shape_type_t = u32;

const SHAPE_COUNT: shape_type_t = 10;
const MAX_SCENES: usize = 10;
const MAX_SCENE_NAME: usize = 64;
const MAX_SHAPES_IN_SCENE: usize = 50;

#[derive(Clone)]
struct Shape {
    type_: shape_type_t,
    name: &'static str,
    art: &'static [&'static str],
    width: i32,
    height: i32,
}

struct Scene {
    name: String,
    shapes: Vec<*const Shape>, // pointer identity semantics
}

thread_local! {
    static SHAPES: RefCell<Vec<Box<Shape>>> = const { RefCell::new(Vec::new()) };
}
thread_local! {
    static SCENES: RefCell<Vec<Box<Scene>>> = const { RefCell::new(Vec::new()) };
}
thread_local! {
    static SCENE_COUNT: Cell<i32> = const { Cell::new(0) };
}

fn print_menu() {
    println!();
    println!("=========================================");
    println!("  ASCII ART DRAWING APPLICATION");
    println!("=========================================");
    println!("1. View all available shapes");
    println!("2. Create new scene");
    println!("3. Add shape to scene");
    println!("4. Remove shape from scene");
    println!("5. View scene");
    println!("6. List all scenes");
    println!("7. Save scene");
    println!("8. Load scene");
    println!("9. Compare two shapes");
    println!("10. Compare two scenes");
    println!("11. Delete scene");
    println!("12. Exit");
    println!("=========================================");
    print!("Choice: ");
    let _ = io::stdout().flush();
}

fn read_line_raw() -> Option<String> {
    let mut s = String::new();
    let n = io::stdin().lock().read_line(&mut s).ok()?;
    if n == 0 {
        return None;
    }
    Some(s)
}

fn trim_newlines(mut s: String) -> String {
    while s.ends_with('\n') || s.ends_with('\r') {
        s.pop();
    }
    s
}

fn parse_choice_from_line(line: &str) -> Option<i32> {
    // Mimic sscanf("%d"): skip leading whitespace, parse optional sign + digits.
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    if i >= bytes.len() {
        return None;
    }
    let mut sign = 1i32;
    if bytes[i] == b'+' {
        i += 1;
    } else if bytes[i] == b'-' {
        sign = -1;
        i += 1;
    }
    let start_digits = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == start_digits {
        return None;
    }
    let num_str = &line[start_digits..i];
    let v = num_str.parse::<i32>().ok()?;
    Some(sign * v)
}

fn read_i32_from_line() -> Result<i32, ()> {
    let s = read_line_raw().ok_or(())?;
    parse_choice_from_line(&s).ok_or(())
}

fn shape_type_name(type_: shape_type_t) -> &'static str {
    match type_ {
        0 => "Tree",
        1 => "Tractor",
        2 => "House",
        3 => "Sun",
        4 => "Cloud",
        5 => "Flower",
        6 => "Car",
        7 => "Star",
        8 => "Heart",
        9 => "Rainbow",
        _ => "Unknown",
    }
}

fn shape_get(type_: shape_type_t) -> *const Shape {
    if type_ >= SHAPE_COUNT {
        return std::ptr::null();
    }
    SHAPES.with(|shapes| {
        let shapes = shapes.borrow();
        shapes
            .get(type_ as usize)
            .map(|b| &**b as *const Shape)
            .unwrap_or(std::ptr::null())
    })
}

fn shape_print(shape: Option<&mut Shape>) {
    if shape.is_none() {
        println!("(null shape)");
        return;
    }
    let shape = shape.unwrap();
    println!("{}:", shape.name);
    for i in 0..shape.height.max(0) as usize {
        let line = shape.art.get(i).copied().unwrap_or("");
        println!("{line}");
    }
}

fn shape_equals(s1: Option<&Shape>, s2: Option<&Shape>) -> i32 {
    let p1 = s1.map(|s| s as *const Shape).unwrap_or(std::ptr::null());
    let p2 = s2.map(|s| s as *const Shape).unwrap_or(std::ptr::null());
    if p1 == p2 { 1 } else { 0 }
}

fn shape_manager_init() {
    fn mk(
        type_: shape_type_t,
        name: &'static str,
        width: i32,
        height: i32,
        art: &'static [&'static str],
    ) -> Box<Shape> {
        Box::new(Shape {
            type_,
            name,
            art,
            width,
            height,
        })
    }

    let mut v: Vec<Box<Shape>> = Vec::with_capacity(SHAPE_COUNT as usize);
    v.push(mk(
        0,
        "Tree",
        11,
        7,
        &[
            "    /\\    ",
            "   /  \\   ",
            "  /____\\  ",
            "  /    \\  ",
            " /______\\ ",
            "    ||    ",
            "    ||    ",
        ],
    ));
    v.push(mk(
        1,
        "Tractor",
        20,
        6,
        &[
            "      ________     ",
            "     |        |___ ",
            "     |  []  []|   |",
            "  ___|________|___|",
            " /  o        o   \\",
            "|___|        |___| ",
        ],
    ));
    v.push(mk(
        2,
        "House",
        13,
        7,
        &[
            "     /\\     ",
            "    /  \\    ",
            "   /____\\   ",
            "   |    |   ",
            "   | [] |   ",
            "   |    |   ",
            "   |____|   ",
        ],
    ));
    v.push(mk(
        3,
        "Sun",
        11,
        7,
        &[
            "  \\  |  / ",
            "   \\ | /  ",
            "--- (@) ---",
            "   / | \\  ",
            "  /  |  \\ ",
            "          ",
            "          ",
        ],
    ));
    v.push(mk(
        4,
        "Cloud",
        16,
        4,
        &[
            "   _____       ",
            "  /     \\_    ",
            " /  ___  _\\  ",
            "(__/   \\_)   ",
        ],
    ));
    v.push(mk(
        5,
        "Flower",
        9,
        7,
        &[
            "  \\|/  ",
            " -(@)- ",
            "  /|\\  ",
            "   |   ",
            "   |   ",
            "  / \\  ",
            " /   \\ ",
        ],
    ));
    v.push(mk(
        6,
        "Car",
        16,
        4,
        &[
            "  ____       ",
            " /|_||_\\____ ",
            "( o     o  ) ",
            " -----------  ",
        ],
    ));
    v.push(mk(
        7,
        "Star",
        9,
        5,
        &["    *    ", "   ***   ", "  *****  ", " ******* ", "*********"],
    ));
    v.push(mk(
        8,
        "Heart",
        11,
        6,
        &[
            " *** ***  ",
            "*********  ",
            "*********  ",
            " ******* ",
            "  *****  ",
            "   ***   ",
        ],
    ));
    v.push(mk(
        9,
        "Rainbow",
        21,
        5,
        &[
            "      _______      ",
            "    /         \\    ",
            "   /           \\   ",
            "  /             \\  ",
            " /               \\ ",
        ],
    ));

    SHAPES.with(|shapes| *shapes.borrow_mut() = v);
}

fn shape_manager_cleanup() {
    SHAPES.with(|shapes| shapes.borrow_mut().clear());
}

fn scene_create_internal(name: &str) -> Option<Box<Scene>> {
    let mut nm = if name.is_empty() {
        "Untitled Scene".to_string()
    } else {
        name.to_string()
    };
    if nm.len() >= MAX_SCENE_NAME {
        nm.truncate(MAX_SCENE_NAME - 1);
    }
    Some(Box::new(Scene {
        name: nm,
        shapes: Vec::new(),
    }))
}

fn scene_add_shape(scene: &mut Scene, shape: *const Shape) -> i32 {
    if shape.is_null() {
        return -1;
    }
    if scene.shapes.len() >= MAX_SHAPES_IN_SCENE {
        eprintln!("Error: Scene is full");
        return -1;
    }
    scene.shapes.push(shape);
    0
}

fn scene_remove_shape(scene: &mut Scene, index: i32) -> i32 {
    if index < 0 {
        return -1;
    }
    let idx = index as usize;
    if idx >= scene.shapes.len() {
        return -1;
    }
    scene.shapes.remove(idx);
    0
}

fn scene_print(scene: Option<&mut Scene>) {
    if scene.is_none() {
        println!("(null scene)");
        return;
    }
    let scene = scene.unwrap();
    print!("\n=== Scene: {} ===\n", scene.name);
    print!("Contains {} shape(s)\n\n", scene.shapes.len());
    for (i, &sp) in scene.shapes.iter().enumerate() {
        println!("Shape #{}:", i + 1);
        let mut_ref = unsafe { (sp as *mut Shape).as_mut() };
        shape_print(mut_ref);
        println!();
    }
}

fn scene_list_shapes(scene: Option<&mut Scene>) {
    if scene.is_none() {
        println!("(null scene)");
        return;
    }
    let scene = scene.unwrap();
    print!("\nScene: {}\n", scene.name);
    println!("Shapes ({}):", scene.shapes.len());
    for (i, &sp) in scene.shapes.iter().enumerate() {
        let name = unsafe { sp.as_ref().map(|s| s.name).unwrap_or("(null)") };
        println!("  {}. {} (ptr: {:#x})", i + 1, name, sp as usize);
    }
}

fn scene_equals(s1: Option<&Scene>, s2: Option<&Scene>) -> i32 {
    let (Some(s1), Some(s2)) = (s1, s2) else { return 0 };
    if s1.shapes.len() != s2.shapes.len() {
        return 0;
    }
    let mut matched = vec![false; s2.shapes.len()];
    for &a in &s1.shapes {
        let mut found = false;
        for (j, &b) in s2.shapes.iter().enumerate() {
            if !matched[j] && a == b {
                matched[j] = true;
                found = true;
                break;
            }
        }
        if !found {
            return 0;
        }
    }
    1
}

fn scene_save_internal(scene: Option<&mut Scene>, filename: &str) -> i32 {
    if scene.is_none() || filename.is_empty() {
        return -1;
    }
    let scene = scene.unwrap();

    let file = std::fs::File::create(filename);
    let mut file = match file {
        Ok(f) => io::BufWriter::new(f),
        Err(_) => {
            eprintln!("Error: Could not open file '{}' for writing", filename);
            return -1;
        }
    };

    if writeln!(file, "{}", scene.name).is_err() {
        return -1;
    }
    if writeln!(file, "{}", scene.shapes.len() as i32).is_err() {
        return -1;
    }
    for &sp in &scene.shapes {
        let t = unsafe { sp.as_ref().map(|s| s.type_).unwrap_or(0) };
        if writeln!(file, "{}", t as i32).is_err() {
            return -1;
        }
    }
    if file.flush().is_err() {
        return -1;
    }

    println!("Scene saved to '{}'", filename);
    0
}

fn scene_load_internal(filename: &str) -> Option<Box<Scene>> {
    if filename.is_empty() {
        return None;
    }
    let file = std::fs::File::open(filename);
    let file = match file {
        Ok(f) => io::BufReader::new(f),
        Err(_) => {
            eprintln!("Error: Could not open file '{}' for reading", filename);
            return None;
        }
    };

    let mut lines = file.lines();

    let name_line = match lines.next() {
        Some(Ok(s)) => s,
        _ => return None,
    };
    let mut scene = scene_create_internal(&trim_newlines(name_line))?;

    let shape_count_line = match lines.next() {
        Some(Ok(s)) => s,
        _ => return None,
    };
    let shape_count: i32 = parse_choice_from_line(&shape_count_line)
        .or_else(|| shape_count_line.trim().parse::<i32>().ok())?;

    for _ in 0..shape_count.max(0) {
        let type_line = match lines.next() {
            Some(Ok(s)) => s,
            _ => return None,
        };
        let type_i32: i32 = parse_choice_from_line(&type_line)
            .or_else(|| type_line.trim().parse::<i32>().ok())?;
        if type_i32 < 0 {
            continue;
        }
        let sp = shape_get(type_i32 as u32);
        if !sp.is_null() {
            let _ = scene_add_shape(&mut scene, sp);
        }
    }

    println!("Scene loaded from '{}'", filename);
    Some(scene)
}

fn view_all_shapes() {
    print!("\n=== Available Shapes ===\n");
    for i in 0..(SHAPE_COUNT as i32) {
        print!("\n{}. ", i + 1);
        let sp = shape_get(i as u32);
        let mut_ref = unsafe { (sp as *mut Shape).as_mut() };
        shape_print(mut_ref);
    }
}

fn create_new_scene() {
    let count = SCENE_COUNT.with(|c| c.get()) as usize;
    if count >= MAX_SCENES {
        println!("Error: Maximum scenes reached");
        return;
    }

    print!("Enter scene name: ");
    let _ = io::stdout().flush();
    let Some(line) = read_line_raw() else { return };
    let name = trim_newlines(line);

    let scene = match scene_create_internal(&name) {
        Some(s) => s,
        None => {
            println!("Error creating scene");
            return;
        }
    };

    SCENES.with(|scenes| scenes.borrow_mut().push(scene));
    println!("Scene '{}' created (index {})", name, count);
    SCENE_COUNT.with(|c| c.set(c.get() + 1));
}

fn add_shape_to_scene() {
    let sc_count = SCENE_COUNT.with(|c| c.get());
    if sc_count == 0 {
        println!("No scenes available. Create a scene first.");
        return;
    }

    print!("Select scene (0-{}): ", sc_count - 1);
    let _ = io::stdout().flush();
    let scene_idx = match read_i32_from_line() {
        Ok(v) => v,
        Err(_) => {
            println!("Invalid input");
            return;
        }
    };

    if scene_idx < 0 || scene_idx >= sc_count {
        println!("Invalid scene index");
        return;
    }

    print!("\nSelect shape to add:\n");
    for i in 0..(SHAPE_COUNT as i32) {
        println!("{}. {}", i, shape_type_name(i as u32));
    }
    print!("Choice: ");
    let _ = io::stdout().flush();

    let shape_type = match read_i32_from_line() {
        Ok(v) => v,
        Err(_) => {
            println!("Invalid input");
            return;
        }
    };

    if shape_type < 0 || shape_type >= SHAPE_COUNT as i32 {
        println!("Invalid shape type");
        return;
    }

    let shape_ptr = shape_get(shape_type as u32);
    let ok = SCENES.with(|scenes| {
        let mut scenes = scenes.borrow_mut();
        let scene = scenes.get_mut(scene_idx as usize).map(|b| &mut **b);
        match scene {
            Some(sc) => scene_add_shape(sc, shape_ptr) == 0,
            None => false,
        }
    });

    if ok {
        let name = unsafe { shape_ptr.as_ref().map(|s| s.name).unwrap_or("") };
        println!(
            "Shape '{}' added to scene (reusing singleton at {:#x})",
            name,
            shape_ptr as usize
        );
    } else {
        println!("Error adding shape");
    }
}

fn remove_shape_from_scene() {
    let sc_count = SCENE_COUNT.with(|c| c.get());
    if sc_count == 0 {
        println!("No scenes available");
        return;
    }

    print!("Select scene (0-{}): ", sc_count - 1);
    let _ = io::stdout().flush();
    let scene_idx = match read_i32_from_line() {
        Ok(v) => v,
        Err(_) => {
            println!("Invalid input");
            return;
        }
    };

    if scene_idx < 0 || scene_idx >= sc_count {
        println!("Invalid scene index");
        return;
    }

    SCENES.with(|scenes| {
        let mut scenes = scenes.borrow_mut();
        let sc = scenes.get_mut(scene_idx as usize).map(|b| &mut **b);
        scene_list_shapes(sc);
    });

    let shape_count = SCENES.with(|scenes| {
        scenes
            .borrow()
            .get(scene_idx as usize)
            .map(|s| s.shapes.len() as i32)
            .unwrap_or(0)
    });
    if shape_count == 0 {
        println!("Scene is empty");
        return;
    }

    print!("Select shape to remove (1-{}): ", shape_count);
    let _ = io::stdout().flush();
    let shape_idx = match read_i32_from_line() {
        Ok(v) => v,
        Err(_) => {
            println!("Invalid input");
            return;
        }
    };

    let ok = SCENES.with(|scenes| {
        let mut scenes = scenes.borrow_mut();
        let sc = scenes.get_mut(scene_idx as usize).map(|b| &
// ... (truncated) ...
```

**Entity:** SHAPES (thread_local RefCell<Vec<Box<Shape>>>) / shape_get/shape_manager_init/shape_manager_cleanup

**States:** UninitializedOrEmpty, Initialized, CleanedUp

**Transitions:**
- UninitializedOrEmpty -> Initialized via shape_manager_init()
- Initialized -> CleanedUp via shape_manager_cleanup()

**Evidence:** thread_local! static SHAPES: RefCell<Vec<Box<Shape>>> = ... Vec::new() (empty initial state); fn shape_manager_init(): SHAPES.with(|shapes| *shapes.borrow_mut() = v) (populates registry); fn shape_manager_cleanup(): shapes.borrow_mut().clear() (clears registry); fn shape_get(type_): if type_ >= SHAPE_COUNT { return std::ptr::null(); } ... unwrap_or(std::ptr::null()) (null sentinel for invalid/unavailable); view_all_shapes(): let sp = shape_get(...); let mut_ref = unsafe { (sp as *mut Shape).as_mut() }; shape_print(mut_ref) (propagates nullability through unsafe pointer cast)

**Implementation:** Replace raw global access with a manager handle: struct ShapeManager<S> { shapes: Vec<Box<Shape>>, _s: PhantomData<S> }. ShapeManager<Uninit>::init(self) -> ShapeManager<Init>. Expose ShapeId as a newtype (e.g., struct ShapeId(NonZeroU8) or bounded u8) and return &Shape from get(&self, ShapeId) (no null). Scenes store ShapeId (or Rc/Arc) instead of *const Shape, eliminating use-after-cleanup/null protocols.

---

## State Machine Invariants

### 3. Scene contents validity protocol (only pointers to registered Shapes; capacity/index constraints)

**Location**: `/data/test_case/main.rs:1-652`

**Confidence**: high

**Suggested Pattern**: newtype

**Description**: Scene has an implicit state machine driven by its shapes vector length and by validity of the stored shape pointers. scene_add_shape requires a non-null shape pointer and enforces MAX_SHAPES_IN_SCENE at runtime; scene_remove_shape requires an in-bounds index. Additionally, Scene stores Vec<*const Shape> with 'pointer identity semantics', implying an invariant that pointers must come from the SHAPES registry and remain valid for the Scene's lifetime. This is not enforced: any *const Shape (including null or dangling after shape_manager_cleanup) can be inserted, and printing/loading uses unsafe pointer deref paths.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Shape, 4 free function(s); struct Scene, 7 free function(s)

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

use std::cell::{Cell, RefCell};
use std::io::{self, BufRead, Write};

type shape_type_t = u32;

const SHAPE_COUNT: shape_type_t = 10;
const MAX_SCENES: usize = 10;
const MAX_SCENE_NAME: usize = 64;
const MAX_SHAPES_IN_SCENE: usize = 50;

#[derive(Clone)]
struct Shape {
    type_: shape_type_t,
    name: &'static str,
    art: &'static [&'static str],
    width: i32,
    height: i32,
}

struct Scene {
    name: String,
    shapes: Vec<*const Shape>, // pointer identity semantics
}

thread_local! {
    static SHAPES: RefCell<Vec<Box<Shape>>> = const { RefCell::new(Vec::new()) };
}
thread_local! {
    static SCENES: RefCell<Vec<Box<Scene>>> = const { RefCell::new(Vec::new()) };
}
thread_local! {
    static SCENE_COUNT: Cell<i32> = const { Cell::new(0) };
}

fn print_menu() {
    println!();
    println!("=========================================");
    println!("  ASCII ART DRAWING APPLICATION");
    println!("=========================================");
    println!("1. View all available shapes");
    println!("2. Create new scene");
    println!("3. Add shape to scene");
    println!("4. Remove shape from scene");
    println!("5. View scene");
    println!("6. List all scenes");
    println!("7. Save scene");
    println!("8. Load scene");
    println!("9. Compare two shapes");
    println!("10. Compare two scenes");
    println!("11. Delete scene");
    println!("12. Exit");
    println!("=========================================");
    print!("Choice: ");
    let _ = io::stdout().flush();
}

fn read_line_raw() -> Option<String> {
    let mut s = String::new();
    let n = io::stdin().lock().read_line(&mut s).ok()?;
    if n == 0 {
        return None;
    }
    Some(s)
}

fn trim_newlines(mut s: String) -> String {
    while s.ends_with('\n') || s.ends_with('\r') {
        s.pop();
    }
    s
}

fn parse_choice_from_line(line: &str) -> Option<i32> {
    // Mimic sscanf("%d"): skip leading whitespace, parse optional sign + digits.
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    if i >= bytes.len() {
        return None;
    }
    let mut sign = 1i32;
    if bytes[i] == b'+' {
        i += 1;
    } else if bytes[i] == b'-' {
        sign = -1;
        i += 1;
    }
    let start_digits = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == start_digits {
        return None;
    }
    let num_str = &line[start_digits..i];
    let v = num_str.parse::<i32>().ok()?;
    Some(sign * v)
}

fn read_i32_from_line() -> Result<i32, ()> {
    let s = read_line_raw().ok_or(())?;
    parse_choice_from_line(&s).ok_or(())
}

fn shape_type_name(type_: shape_type_t) -> &'static str {
    match type_ {
        0 => "Tree",
        1 => "Tractor",
        2 => "House",
        3 => "Sun",
        4 => "Cloud",
        5 => "Flower",
        6 => "Car",
        7 => "Star",
        8 => "Heart",
        9 => "Rainbow",
        _ => "Unknown",
    }
}

fn shape_get(type_: shape_type_t) -> *const Shape {
    if type_ >= SHAPE_COUNT {
        return std::ptr::null();
    }
    SHAPES.with(|shapes| {
        let shapes = shapes.borrow();
        shapes
            .get(type_ as usize)
            .map(|b| &**b as *const Shape)
            .unwrap_or(std::ptr::null())
    })
}

fn shape_print(shape: Option<&mut Shape>) {
    if shape.is_none() {
        println!("(null shape)");
        return;
    }
    let shape = shape.unwrap();
    println!("{}:", shape.name);
    for i in 0..shape.height.max(0) as usize {
        let line = shape.art.get(i).copied().unwrap_or("");
        println!("{line}");
    }
}

fn shape_equals(s1: Option<&Shape>, s2: Option<&Shape>) -> i32 {
    let p1 = s1.map(|s| s as *const Shape).unwrap_or(std::ptr::null());
    let p2 = s2.map(|s| s as *const Shape).unwrap_or(std::ptr::null());
    if p1 == p2 { 1 } else { 0 }
}

fn shape_manager_init() {
    fn mk(
        type_: shape_type_t,
        name: &'static str,
        width: i32,
        height: i32,
        art: &'static [&'static str],
    ) -> Box<Shape> {
        Box::new(Shape {
            type_,
            name,
            art,
            width,
            height,
        })
    }

    let mut v: Vec<Box<Shape>> = Vec::with_capacity(SHAPE_COUNT as usize);
    v.push(mk(
        0,
        "Tree",
        11,
        7,
        &[
            "    /\\    ",
            "   /  \\   ",
            "  /____\\  ",
            "  /    \\  ",
            " /______\\ ",
            "    ||    ",
            "    ||    ",
        ],
    ));
    v.push(mk(
        1,
        "Tractor",
        20,
        6,
        &[
            "      ________     ",
            "     |        |___ ",
            "     |  []  []|   |",
            "  ___|________|___|",
            " /  o        o   \\",
            "|___|        |___| ",
        ],
    ));
    v.push(mk(
        2,
        "House",
        13,
        7,
        &[
            "     /\\     ",
            "    /  \\    ",
            "   /____\\   ",
            "   |    |   ",
            "   | [] |   ",
            "   |    |   ",
            "   |____|   ",
        ],
    ));
    v.push(mk(
        3,
        "Sun",
        11,
        7,
        &[
            "  \\  |  / ",
            "   \\ | /  ",
            "--- (@) ---",
            "   / | \\  ",
            "  /  |  \\ ",
            "          ",
            "          ",
        ],
    ));
    v.push(mk(
        4,
        "Cloud",
        16,
        4,
        &[
            "   _____       ",
            "  /     \\_    ",
            " /  ___  _\\  ",
            "(__/   \\_)   ",
        ],
    ));
    v.push(mk(
        5,
        "Flower",
        9,
        7,
        &[
            "  \\|/  ",
            " -(@)- ",
            "  /|\\  ",
            "   |   ",
            "   |   ",
            "  / \\  ",
            " /   \\ ",
        ],
    ));
    v.push(mk(
        6,
        "Car",
        16,
        4,
        &[
            "  ____       ",
            " /|_||_\\____ ",
            "( o     o  ) ",
            " -----------  ",
        ],
    ));
    v.push(mk(
        7,
        "Star",
        9,
        5,
        &["    *    ", "   ***   ", "  *****  ", " ******* ", "*********"],
    ));
    v.push(mk(
        8,
        "Heart",
        11,
        6,
        &[
            " *** ***  ",
            "*********  ",
            "*********  ",
            " ******* ",
            "  *****  ",
            "   ***   ",
        ],
    ));
    v.push(mk(
        9,
        "Rainbow",
        21,
        5,
        &[
            "      _______      ",
            "    /         \\    ",
            "   /           \\   ",
            "  /             \\  ",
            " /               \\ ",
        ],
    ));

    SHAPES.with(|shapes| *shapes.borrow_mut() = v);
}

fn shape_manager_cleanup() {
    SHAPES.with(|shapes| shapes.borrow_mut().clear());
}

fn scene_create_internal(name: &str) -> Option<Box<Scene>> {
    let mut nm = if name.is_empty() {
        "Untitled Scene".to_string()
    } else {
        name.to_string()
    };
    if nm.len() >= MAX_SCENE_NAME {
        nm.truncate(MAX_SCENE_NAME - 1);
    }
    Some(Box::new(Scene {
        name: nm,
        shapes: Vec::new(),
    }))
}

fn scene_add_shape(scene: &mut Scene, shape: *const Shape) -> i32 {
    if shape.is_null() {
        return -1;
    }
    if scene.shapes.len() >= MAX_SHAPES_IN_SCENE {
        eprintln!("Error: Scene is full");
        return -1;
    }
    scene.shapes.push(shape);
    0
}

fn scene_remove_shape(scene: &mut Scene, index: i32) -> i32 {
    if index < 0 {
        return -1;
    }
    let idx = index as usize;
    if idx >= scene.shapes.len() {
        return -1;
    }
    scene.shapes.remove(idx);
    0
}

fn scene_print(scene: Option<&mut Scene>) {
    if scene.is_none() {
        println!("(null scene)");
        return;
    }
    let scene = scene.unwrap();
    print!("\n=== Scene: {} ===\n", scene.name);
    print!("Contains {} shape(s)\n\n", scene.shapes.len());
    for (i, &sp) in scene.shapes.iter().enumerate() {
        println!("Shape #{}:", i + 1);
        let mut_ref = unsafe { (sp as *mut Shape).as_mut() };
        shape_print(mut_ref);
        println!();
    }
}

fn scene_list_shapes(scene: Option<&mut Scene>) {
    if scene.is_none() {
        println!("(null scene)");
        return;
    }
    let scene = scene.unwrap();
    print!("\nScene: {}\n", scene.name);
    println!("Shapes ({}):", scene.shapes.len());
    for (i, &sp) in scene.shapes.iter().enumerate() {
        let name = unsafe { sp.as_ref().map(|s| s.name).unwrap_or("(null)") };
        println!("  {}. {} (ptr: {:#x})", i + 1, name, sp as usize);
    }
}

fn scene_equals(s1: Option<&Scene>, s2: Option<&Scene>) -> i32 {
    let (Some(s1), Some(s2)) = (s1, s2) else { return 0 };
    if s1.shapes.len() != s2.shapes.len() {
        return 0;
    }
    let mut matched = vec![false; s2.shapes.len()];
    for &a in &s1.shapes {
        let mut found = false;
        for (j, &b) in s2.shapes.iter().enumerate() {
            if !matched[j] && a == b {
                matched[j] = true;
                found = true;
                break;
            }
        }
        if !found {
            return 0;
        }
    }
    1
}

fn scene_save_internal(scene: Option<&mut Scene>, filename: &str) -> i32 {
    if scene.is_none() || filename.is_empty() {
        return -1;
    }
    let scene = scene.unwrap();

    let file = std::fs::File::create(filename);
    let mut file = match file {
        Ok(f) => io::BufWriter::new(f),
        Err(_) => {
            eprintln!("Error: Could not open file '{}' for writing", filename);
            return -1;
        }
    };

    if writeln!(file, "{}", scene.name).is_err() {
        return -1;
    }
    if writeln!(file, "{}", scene.shapes.len() as i32).is_err() {
        return -1;
    }
    for &sp in &scene.shapes {
        let t = unsafe { sp.as_ref().map(|s| s.type_).unwrap_or(0) };
        if writeln!(file, "{}", t as i32).is_err() {
            return -1;
        }
    }
    if file.flush().is_err() {
        return -1;
    }

    println!("Scene saved to '{}'", filename);
    0
}

fn scene_load_internal(filename: &str) -> Option<Box<Scene>> {
    if filename.is_empty() {
        return None;
    }
    let file = std::fs::File::open(filename);
    let file = match file {
        Ok(f) => io::BufReader::new(f),
        Err(_) => {
            eprintln!("Error: Could not open file '{}' for reading", filename);
            return None;
        }
    };

    let mut lines = file.lines();

    let name_line = match lines.next() {
        Some(Ok(s)) => s,
        _ => return None,
    };
    let mut scene = scene_create_internal(&trim_newlines(name_line))?;

    let shape_count_line = match lines.next() {
        Some(Ok(s)) => s,
        _ => return None,
    };
    let shape_count: i32 = parse_choice_from_line(&shape_count_line)
        .or_else(|| shape_count_line.trim().parse::<i32>().ok())?;

    for _ in 0..shape_count.max(0) {
        let type_line = match lines.next() {
            Some(Ok(s)) => s,
            _ => return None,
        };
        let type_i32: i32 = parse_choice_from_line(&type_line)
            .or_else(|| type_line.trim().parse::<i32>().ok())?;
        if type_i32 < 0 {
            continue;
        }
        let sp = shape_get(type_i32 as u32);
        if !sp.is_null() {
            let _ = scene_add_shape(&mut scene, sp);
        }
    }

    println!("Scene loaded from '{}'", filename);
    Some(scene)
}

fn view_all_shapes() {
    print!("\n=== Available Shapes ===\n");
    for i in 0..(SHAPE_COUNT as i32) {
        print!("\n{}. ", i + 1);
        let sp = shape_get(i as u32);
        let mut_ref = unsafe { (sp as *mut Shape).as_mut() };
        shape_print(mut_ref);
    }
}

fn create_new_scene() {
    let count = SCENE_COUNT.with(|c| c.get()) as usize;
    if count >= MAX_SCENES {
        println!("Error: Maximum scenes reached");
        return;
    }

    print!("Enter scene name: ");
    let _ = io::stdout().flush();
    let Some(line) = read_line_raw() else { return };
    let name = trim_newlines(line);

    let scene = match scene_create_internal(&name) {
        Some(s) => s,
        None => {
            println!("Error creating scene");
            return;
        }
    };

    SCENES.with(|scenes| scenes.borrow_mut().push(scene));
    println!("Scene '{}' created (index {})", name, count);
    SCENE_COUNT.with(|c| c.set(c.get() + 1));
}

fn add_shape_to_scene() {
    let sc_count = SCENE_COUNT.with(|c| c.get());
    if sc_count == 0 {
        println!("No scenes available. Create a scene first.");
        return;
    }

    print!("Select scene (0-{}): ", sc_count - 1);
    let _ = io::stdout().flush();
    let scene_idx = match read_i32_from_line() {
        Ok(v) => v,
        Err(_) => {
            println!("Invalid input");
            return;
        }
    };

    if scene_idx < 0 || scene_idx >= sc_count {
        println!("Invalid scene index");
        return;
    }

    print!("\nSelect shape to add:\n");
    for i in 0..(SHAPE_COUNT as i32) {
        println!("{}. {}", i, shape_type_name(i as u32));
    }
    print!("Choice: ");
    let _ = io::stdout().flush();

    let shape_type = match read_i32_from_line() {
        Ok(v) => v,
        Err(_) => {
            println!("Invalid input");
            return;
        }
    };

    if shape_type < 0 || shape_type >= SHAPE_COUNT as i32 {
        println!("Invalid shape type");
        return;
    }

    let shape_ptr = shape_get(shape_type as u32);
    let ok = SCENES.with(|scenes| {
        let mut scenes = scenes.borrow_mut();
        let scene = scenes.get_mut(scene_idx as usize).map(|b| &mut **b);
        match scene {
            Some(sc) => scene_add_shape(sc, shape_ptr) == 0,
            None => false,
        }
    });

    if ok {
        let name = unsafe { shape_ptr.as_ref().map(|s| s.name).unwrap_or("") };
        println!(
            "Shape '{}' added to scene (reusing singleton at {:#x})",
            name,
            shape_ptr as usize
        );
    } else {
        println!("Error adding shape");
    }
}

fn remove_shape_from_scene() {
    let sc_count = SCENE_COUNT.with(|c| c.get());
    if sc_count == 0 {
        println!("No scenes available");
        return;
    }

    print!("Select scene (0-{}): ", sc_count - 1);
    let _ = io::stdout().flush();
    let scene_idx = match read_i32_from_line() {
        Ok(v) => v,
        Err(_) => {
            println!("Invalid input");
            return;
        }
    };

    if scene_idx < 0 || scene_idx >= sc_count {
        println!("Invalid scene index");
        return;
    }

    SCENES.with(|scenes| {
        let mut scenes = scenes.borrow_mut();
        let sc = scenes.get_mut(scene_idx as usize).map(|b| &mut **b);
        scene_list_shapes(sc);
    });

    let shape_count = SCENES.with(|scenes| {
        scenes
            .borrow()
            .get(scene_idx as usize)
            .map(|s| s.shapes.len() as i32)
            .unwrap_or(0)
    });
    if shape_count == 0 {
        println!("Scene is empty");
        return;
    }

    print!("Select shape to remove (1-{}): ", shape_count);
    let _ = io::stdout().flush();
    let shape_idx = match read_i32_from_line() {
        Ok(v) => v,
        Err(_) => {
            println!("Invalid input");
            return;
        }
    };

    let ok = SCENES.with(|scenes| {
        let mut scenes = scenes.borrow_mut();
        let sc = scenes.get_mut(scene_idx as usize).map(|b| &
// ... (truncated) ...
```

**Entity:** Scene

**States:** Empty, NonEmptyNotFull, Full

**Transitions:**
- Empty -> NonEmptyNotFull via scene_add_shape()
- NonEmptyNotFull -> Full via repeated scene_add_shape() until MAX_SHAPES_IN_SCENE
- Full -> NonEmptyNotFull via scene_remove_shape()
- NonEmptyNotFull -> Empty via scene_remove_shape() until len == 0

**Evidence:** struct Scene { shapes: Vec<*const Shape>, // pointer identity semantics } (raw pointer storage + identity requirement in comment); fn scene_add_shape(scene, shape): if shape.is_null() { return -1; } (non-null precondition); fn scene_add_shape: if scene.shapes.len() >= MAX_SHAPES_IN_SCENE { eprintln!("Error: Scene is full"); return -1; } (capacity state runtime-checked); fn scene_remove_shape(scene, index): if idx >= scene.shapes.len() { return -1; } (index validity runtime-checked); scene_print(): let mut_ref = unsafe { (sp as *mut Shape).as_mut() }; shape_print(mut_ref) (unsafe deref of stored pointer); scene_list_shapes(): let name = unsafe { sp.as_ref().map(|s| s.name).unwrap_or("(null)") } (assumes pointer validity to display)

**Implementation:** Store ShapeId (bounded) instead of *const Shape: struct ShapeId(u8) with TryFrom<i32>/TryFrom<u32> ensuring < SHAPE_COUNT. Scene then holds Vec<ShapeId> (and capacity can be expressed via an ArrayVec/const-generic wrapper). Access to actual Shape goes through ShapeManager::get(&self, ShapeId) -> &Shape, removing null/dangling pointer possibilities and making add/remove preconditions type-directed.

---

## Precondition Invariants

### 1. Scene raw-pointer shape membership protocol (borrowed shapes must outlive Scene)

**Location**: `/data/test_case/main.rs:1-9`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: Scene stores `Vec<*const Shape>` with "pointer identity semantics". This implies an implicit safety invariant: every pointer in `shapes` must refer to a live `Shape` for as long as the `Scene` uses it. The type system cannot enforce lifetimes/validity of `*const Shape`, so the code relies on external discipline to ensure Shapes outlive the Scene (or at least outlive any use of those pointers). This also implicitly requires the pointers to be non-null and properly aligned, neither of which is represented in the type.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Shape, 4 free function(s); 21 free function(s)

    height: i32,
}

struct Scene {
    name: String,
    shapes: Vec<*const Shape>, // pointer identity semantics
}

```

**Entity:** Scene

**States:** Valid (all pointers point to live Shapes), Invalid/Dangling (one or more pointers no longer valid)

**Transitions:**
- Valid -> Invalid/Dangling when a referenced Shape is dropped/moved while its pointer remains in Scene

**Evidence:** field `Scene::shapes: Vec<*const Shape>` uses raw pointers (no lifetime tracking); comment `// pointer identity semantics` indicates the design relies on pointer identity rather than owned values/borrows

**Implementation:** Avoid raw pointers in the public state: (a) store indices/IDs into an owning arena (e.g., `SlotMap`, `generational_arena`) and make Scene hold `ShapeId` newtypes; or (b) store references with lifetimes `Vec<&'a Shape>` if borrowing is intended; or (c) store `NonNull<Shape>` plus an owning handle/capability token that proves the allocation stays alive (arena handle) for the Scene's lifetime.

---

## Protocol Invariants

### 4. Scene registry protocol (SCENE_COUNT must mirror SCENES length; indices must be in-range)

**Location**: `/data/test_case/main.rs:1-652`

**Confidence**: medium

**Suggested Pattern**: capability

**Description**: The module maintains two separate sources of truth for the number of scenes: SCENE_COUNT and SCENES.len(). Many operations use SCENE_COUNT to validate indices and enforce MAX_SCENES, then index into SCENES. This creates a latent invariant that SCENE_COUNT must always equal SCENES.borrow().len() and must stay within [0, MAX_SCENES]. The type system does not enforce this coupling; any missed update (e.g., on deletion paths or failed inserts) could desynchronize them and cause out-of-bounds access patterns or incorrect UI validation.

**Evidence**:

```rust
// Note: Other parts of this module contain: struct Shape, 4 free function(s); struct Scene, 7 free function(s)

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

use std::cell::{Cell, RefCell};
use std::io::{self, BufRead, Write};

type shape_type_t = u32;

const SHAPE_COUNT: shape_type_t = 10;
const MAX_SCENES: usize = 10;
const MAX_SCENE_NAME: usize = 64;
const MAX_SHAPES_IN_SCENE: usize = 50;

#[derive(Clone)]
struct Shape {
    type_: shape_type_t,
    name: &'static str,
    art: &'static [&'static str],
    width: i32,
    height: i32,
}

struct Scene {
    name: String,
    shapes: Vec<*const Shape>, // pointer identity semantics
}

thread_local! {
    static SHAPES: RefCell<Vec<Box<Shape>>> = const { RefCell::new(Vec::new()) };
}
thread_local! {
    static SCENES: RefCell<Vec<Box<Scene>>> = const { RefCell::new(Vec::new()) };
}
thread_local! {
    static SCENE_COUNT: Cell<i32> = const { Cell::new(0) };
}

fn print_menu() {
    println!();
    println!("=========================================");
    println!("  ASCII ART DRAWING APPLICATION");
    println!("=========================================");
    println!("1. View all available shapes");
    println!("2. Create new scene");
    println!("3. Add shape to scene");
    println!("4. Remove shape from scene");
    println!("5. View scene");
    println!("6. List all scenes");
    println!("7. Save scene");
    println!("8. Load scene");
    println!("9. Compare two shapes");
    println!("10. Compare two scenes");
    println!("11. Delete scene");
    println!("12. Exit");
    println!("=========================================");
    print!("Choice: ");
    let _ = io::stdout().flush();
}

fn read_line_raw() -> Option<String> {
    let mut s = String::new();
    let n = io::stdin().lock().read_line(&mut s).ok()?;
    if n == 0 {
        return None;
    }
    Some(s)
}

fn trim_newlines(mut s: String) -> String {
    while s.ends_with('\n') || s.ends_with('\r') {
        s.pop();
    }
    s
}

fn parse_choice_from_line(line: &str) -> Option<i32> {
    // Mimic sscanf("%d"): skip leading whitespace, parse optional sign + digits.
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    if i >= bytes.len() {
        return None;
    }
    let mut sign = 1i32;
    if bytes[i] == b'+' {
        i += 1;
    } else if bytes[i] == b'-' {
        sign = -1;
        i += 1;
    }
    let start_digits = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == start_digits {
        return None;
    }
    let num_str = &line[start_digits..i];
    let v = num_str.parse::<i32>().ok()?;
    Some(sign * v)
}

fn read_i32_from_line() -> Result<i32, ()> {
    let s = read_line_raw().ok_or(())?;
    parse_choice_from_line(&s).ok_or(())
}

fn shape_type_name(type_: shape_type_t) -> &'static str {
    match type_ {
        0 => "Tree",
        1 => "Tractor",
        2 => "House",
        3 => "Sun",
        4 => "Cloud",
        5 => "Flower",
        6 => "Car",
        7 => "Star",
        8 => "Heart",
        9 => "Rainbow",
        _ => "Unknown",
    }
}

fn shape_get(type_: shape_type_t) -> *const Shape {
    if type_ >= SHAPE_COUNT {
        return std::ptr::null();
    }
    SHAPES.with(|shapes| {
        let shapes = shapes.borrow();
        shapes
            .get(type_ as usize)
            .map(|b| &**b as *const Shape)
            .unwrap_or(std::ptr::null())
    })
}

fn shape_print(shape: Option<&mut Shape>) {
    if shape.is_none() {
        println!("(null shape)");
        return;
    }
    let shape = shape.unwrap();
    println!("{}:", shape.name);
    for i in 0..shape.height.max(0) as usize {
        let line = shape.art.get(i).copied().unwrap_or("");
        println!("{line}");
    }
}

fn shape_equals(s1: Option<&Shape>, s2: Option<&Shape>) -> i32 {
    let p1 = s1.map(|s| s as *const Shape).unwrap_or(std::ptr::null());
    let p2 = s2.map(|s| s as *const Shape).unwrap_or(std::ptr::null());
    if p1 == p2 { 1 } else { 0 }
}

fn shape_manager_init() {
    fn mk(
        type_: shape_type_t,
        name: &'static str,
        width: i32,
        height: i32,
        art: &'static [&'static str],
    ) -> Box<Shape> {
        Box::new(Shape {
            type_,
            name,
            art,
            width,
            height,
        })
    }

    let mut v: Vec<Box<Shape>> = Vec::with_capacity(SHAPE_COUNT as usize);
    v.push(mk(
        0,
        "Tree",
        11,
        7,
        &[
            "    /\\    ",
            "   /  \\   ",
            "  /____\\  ",
            "  /    \\  ",
            " /______\\ ",
            "    ||    ",
            "    ||    ",
        ],
    ));
    v.push(mk(
        1,
        "Tractor",
        20,
        6,
        &[
            "      ________     ",
            "     |        |___ ",
            "     |  []  []|   |",
            "  ___|________|___|",
            " /  o        o   \\",
            "|___|        |___| ",
        ],
    ));
    v.push(mk(
        2,
        "House",
        13,
        7,
        &[
            "     /\\     ",
            "    /  \\    ",
            "   /____\\   ",
            "   |    |   ",
            "   | [] |   ",
            "   |    |   ",
            "   |____|   ",
        ],
    ));
    v.push(mk(
        3,
        "Sun",
        11,
        7,
        &[
            "  \\  |  / ",
            "   \\ | /  ",
            "--- (@) ---",
            "   / | \\  ",
            "  /  |  \\ ",
            "          ",
            "          ",
        ],
    ));
    v.push(mk(
        4,
        "Cloud",
        16,
        4,
        &[
            "   _____       ",
            "  /     \\_    ",
            " /  ___  _\\  ",
            "(__/   \\_)   ",
        ],
    ));
    v.push(mk(
        5,
        "Flower",
        9,
        7,
        &[
            "  \\|/  ",
            " -(@)- ",
            "  /|\\  ",
            "   |   ",
            "   |   ",
            "  / \\  ",
            " /   \\ ",
        ],
    ));
    v.push(mk(
        6,
        "Car",
        16,
        4,
        &[
            "  ____       ",
            " /|_||_\\____ ",
            "( o     o  ) ",
            " -----------  ",
        ],
    ));
    v.push(mk(
        7,
        "Star",
        9,
        5,
        &["    *    ", "   ***   ", "  *****  ", " ******* ", "*********"],
    ));
    v.push(mk(
        8,
        "Heart",
        11,
        6,
        &[
            " *** ***  ",
            "*********  ",
            "*********  ",
            " ******* ",
            "  *****  ",
            "   ***   ",
        ],
    ));
    v.push(mk(
        9,
        "Rainbow",
        21,
        5,
        &[
            "      _______      ",
            "    /         \\    ",
            "   /           \\   ",
            "  /             \\  ",
            " /               \\ ",
        ],
    ));

    SHAPES.with(|shapes| *shapes.borrow_mut() = v);
}

fn shape_manager_cleanup() {
    SHAPES.with(|shapes| shapes.borrow_mut().clear());
}

fn scene_create_internal(name: &str) -> Option<Box<Scene>> {
    let mut nm = if name.is_empty() {
        "Untitled Scene".to_string()
    } else {
        name.to_string()
    };
    if nm.len() >= MAX_SCENE_NAME {
        nm.truncate(MAX_SCENE_NAME - 1);
    }
    Some(Box::new(Scene {
        name: nm,
        shapes: Vec::new(),
    }))
}

fn scene_add_shape(scene: &mut Scene, shape: *const Shape) -> i32 {
    if shape.is_null() {
        return -1;
    }
    if scene.shapes.len() >= MAX_SHAPES_IN_SCENE {
        eprintln!("Error: Scene is full");
        return -1;
    }
    scene.shapes.push(shape);
    0
}

fn scene_remove_shape(scene: &mut Scene, index: i32) -> i32 {
    if index < 0 {
        return -1;
    }
    let idx = index as usize;
    if idx >= scene.shapes.len() {
        return -1;
    }
    scene.shapes.remove(idx);
    0
}

fn scene_print(scene: Option<&mut Scene>) {
    if scene.is_none() {
        println!("(null scene)");
        return;
    }
    let scene = scene.unwrap();
    print!("\n=== Scene: {} ===\n", scene.name);
    print!("Contains {} shape(s)\n\n", scene.shapes.len());
    for (i, &sp) in scene.shapes.iter().enumerate() {
        println!("Shape #{}:", i + 1);
        let mut_ref = unsafe { (sp as *mut Shape).as_mut() };
        shape_print(mut_ref);
        println!();
    }
}

fn scene_list_shapes(scene: Option<&mut Scene>) {
    if scene.is_none() {
        println!("(null scene)");
        return;
    }
    let scene = scene.unwrap();
    print!("\nScene: {}\n", scene.name);
    println!("Shapes ({}):", scene.shapes.len());
    for (i, &sp) in scene.shapes.iter().enumerate() {
        let name = unsafe { sp.as_ref().map(|s| s.name).unwrap_or("(null)") };
        println!("  {}. {} (ptr: {:#x})", i + 1, name, sp as usize);
    }
}

fn scene_equals(s1: Option<&Scene>, s2: Option<&Scene>) -> i32 {
    let (Some(s1), Some(s2)) = (s1, s2) else { return 0 };
    if s1.shapes.len() != s2.shapes.len() {
        return 0;
    }
    let mut matched = vec![false; s2.shapes.len()];
    for &a in &s1.shapes {
        let mut found = false;
        for (j, &b) in s2.shapes.iter().enumerate() {
            if !matched[j] && a == b {
                matched[j] = true;
                found = true;
                break;
            }
        }
        if !found {
            return 0;
        }
    }
    1
}

fn scene_save_internal(scene: Option<&mut Scene>, filename: &str) -> i32 {
    if scene.is_none() || filename.is_empty() {
        return -1;
    }
    let scene = scene.unwrap();

    let file = std::fs::File::create(filename);
    let mut file = match file {
        Ok(f) => io::BufWriter::new(f),
        Err(_) => {
            eprintln!("Error: Could not open file '{}' for writing", filename);
            return -1;
        }
    };

    if writeln!(file, "{}", scene.name).is_err() {
        return -1;
    }
    if writeln!(file, "{}", scene.shapes.len() as i32).is_err() {
        return -1;
    }
    for &sp in &scene.shapes {
        let t = unsafe { sp.as_ref().map(|s| s.type_).unwrap_or(0) };
        if writeln!(file, "{}", t as i32).is_err() {
            return -1;
        }
    }
    if file.flush().is_err() {
        return -1;
    }

    println!("Scene saved to '{}'", filename);
    0
}

fn scene_load_internal(filename: &str) -> Option<Box<Scene>> {
    if filename.is_empty() {
        return None;
    }
    let file = std::fs::File::open(filename);
    let file = match file {
        Ok(f) => io::BufReader::new(f),
        Err(_) => {
            eprintln!("Error: Could not open file '{}' for reading", filename);
            return None;
        }
    };

    let mut lines = file.lines();

    let name_line = match lines.next() {
        Some(Ok(s)) => s,
        _ => return None,
    };
    let mut scene = scene_create_internal(&trim_newlines(name_line))?;

    let shape_count_line = match lines.next() {
        Some(Ok(s)) => s,
        _ => return None,
    };
    let shape_count: i32 = parse_choice_from_line(&shape_count_line)
        .or_else(|| shape_count_line.trim().parse::<i32>().ok())?;

    for _ in 0..shape_count.max(0) {
        let type_line = match lines.next() {
            Some(Ok(s)) => s,
            _ => return None,
        };
        let type_i32: i32 = parse_choice_from_line(&type_line)
            .or_else(|| type_line.trim().parse::<i32>().ok())?;
        if type_i32 < 0 {
            continue;
        }
        let sp = shape_get(type_i32 as u32);
        if !sp.is_null() {
            let _ = scene_add_shape(&mut scene, sp);
        }
    }

    println!("Scene loaded from '{}'", filename);
    Some(scene)
}

fn view_all_shapes() {
    print!("\n=== Available Shapes ===\n");
    for i in 0..(SHAPE_COUNT as i32) {
        print!("\n{}. ", i + 1);
        let sp = shape_get(i as u32);
        let mut_ref = unsafe { (sp as *mut Shape).as_mut() };
        shape_print(mut_ref);
    }
}

fn create_new_scene() {
    let count = SCENE_COUNT.with(|c| c.get()) as usize;
    if count >= MAX_SCENES {
        println!("Error: Maximum scenes reached");
        return;
    }

    print!("Enter scene name: ");
    let _ = io::stdout().flush();
    let Some(line) = read_line_raw() else { return };
    let name = trim_newlines(line);

    let scene = match scene_create_internal(&name) {
        Some(s) => s,
        None => {
            println!("Error creating scene");
            return;
        }
    };

    SCENES.with(|scenes| scenes.borrow_mut().push(scene));
    println!("Scene '{}' created (index {})", name, count);
    SCENE_COUNT.with(|c| c.set(c.get() + 1));
}

fn add_shape_to_scene() {
    let sc_count = SCENE_COUNT.with(|c| c.get());
    if sc_count == 0 {
        println!("No scenes available. Create a scene first.");
        return;
    }

    print!("Select scene (0-{}): ", sc_count - 1);
    let _ = io::stdout().flush();
    let scene_idx = match read_i32_from_line() {
        Ok(v) => v,
        Err(_) => {
            println!("Invalid input");
            return;
        }
    };

    if scene_idx < 0 || scene_idx >= sc_count {
        println!("Invalid scene index");
        return;
    }

    print!("\nSelect shape to add:\n");
    for i in 0..(SHAPE_COUNT as i32) {
        println!("{}. {}", i, shape_type_name(i as u32));
    }
    print!("Choice: ");
    let _ = io::stdout().flush();

    let shape_type = match read_i32_from_line() {
        Ok(v) => v,
        Err(_) => {
            println!("Invalid input");
            return;
        }
    };

    if shape_type < 0 || shape_type >= SHAPE_COUNT as i32 {
        println!("Invalid shape type");
        return;
    }

    let shape_ptr = shape_get(shape_type as u32);
    let ok = SCENES.with(|scenes| {
        let mut scenes = scenes.borrow_mut();
        let scene = scenes.get_mut(scene_idx as usize).map(|b| &mut **b);
        match scene {
            Some(sc) => scene_add_shape(sc, shape_ptr) == 0,
            None => false,
        }
    });

    if ok {
        let name = unsafe { shape_ptr.as_ref().map(|s| s.name).unwrap_or("") };
        println!(
            "Shape '{}' added to scene (reusing singleton at {:#x})",
            name,
            shape_ptr as usize
        );
    } else {
        println!("Error adding shape");
    }
}

fn remove_shape_from_scene() {
    let sc_count = SCENE_COUNT.with(|c| c.get());
    if sc_count == 0 {
        println!("No scenes available");
        return;
    }

    print!("Select scene (0-{}): ", sc_count - 1);
    let _ = io::stdout().flush();
    let scene_idx = match read_i32_from_line() {
        Ok(v) => v,
        Err(_) => {
            println!("Invalid input");
            return;
        }
    };

    if scene_idx < 0 || scene_idx >= sc_count {
        println!("Invalid scene index");
        return;
    }

    SCENES.with(|scenes| {
        let mut scenes = scenes.borrow_mut();
        let sc = scenes.get_mut(scene_idx as usize).map(|b| &mut **b);
        scene_list_shapes(sc);
    });

    let shape_count = SCENES.with(|scenes| {
        scenes
            .borrow()
            .get(scene_idx as usize)
            .map(|s| s.shapes.len() as i32)
            .unwrap_or(0)
    });
    if shape_count == 0 {
        println!("Scene is empty");
        return;
    }

    print!("Select shape to remove (1-{}): ", shape_count);
    let _ = io::stdout().flush();
    let shape_idx = match read_i32_from_line() {
        Ok(v) => v,
        Err(_) => {
            println!("Invalid input");
            return;
        }
    };

    let ok = SCENES.with(|scenes| {
        let mut scenes = scenes.borrow_mut();
        let sc = scenes.get_mut(scene_idx as usize).map(|b| &
// ... (truncated) ...
```

**Entity:** SCENES (thread_local RefCell<Vec<Box<Scene>>>) + SCENE_COUNT (thread_local Cell<i32>)

**States:** NoScenes, HasScenesBelowMax, AtMaxScenes

**Transitions:**
- NoScenes -> HasScenesBelowMax via create_new_scene() (push + SCENE_COUNT++)
- HasScenesBelowMax -> AtMaxScenes when SCENE_COUNT reaches MAX_SCENES (guard in create_new_scene())
- AtMaxScenes -> HasScenesBelowMax via deleting a scene (implied by menu option "Delete scene" though code not shown in snippet)

**Evidence:** thread_local! static SCENES: RefCell<Vec<Box<Scene>>> = ... Vec::new() (scene storage); thread_local! static SCENE_COUNT: Cell<i32> = ... Cell::new(0) (separate count state); create_new_scene(): let count = SCENE_COUNT.with(|c| c.get()) as usize; if count >= MAX_SCENES { ... } (uses SCENE_COUNT for max enforcement); create_new_scene(): SCENES.with(|scenes| scenes.borrow_mut().push(scene)); ... SCENE_COUNT.with(|c| c.set(c.get() + 1)); (manual mirroring update); add_shape_to_scene(): let sc_count = SCENE_COUNT.with(|c| c.get()); ... if scene_idx < 0 || scene_idx >= sc_count { ... } (index validity based on SCENE_COUNT); add_shape_to_scene(): scenes.get_mut(scene_idx as usize) ... (actual indexing into SCENES vector)

**Implementation:** Encapsulate SCENES and its count into a single registry type (e.g., struct SceneRegistry { scenes: Vec<Box<Scene>> }) and remove SCENE_COUNT entirely. Expose SceneHandle/SceneIndex as a newtype returned by create() (capability token) and required by operations like add_shape/remove_shape/view, preventing arbitrary i32 indices and eliminating the need to keep a parallel count in sync.

---


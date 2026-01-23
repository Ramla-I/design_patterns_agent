/// A resource that must be explicitly cleaned up
///
/// INVARIANT: Resources must be released before being dropped.
/// This type uses Drop to ensure cleanup, making it a linear type.
/// The #[must_use] annotation ensures the resource isn't accidentally discarded.
#[must_use = "Resources must be properly released"]
pub struct Resource {
    id: usize,
    allocated: bool,
}

impl Resource {
    /// Allocate a new resource
    ///
    /// INVARIANT: Each resource has a unique ID and must be released
    pub fn allocate(id: usize) -> Self {
        println!("Allocating resource {}", id);
        Resource {
            id,
            allocated: true,
        }
    }

    /// Use the resource
    ///
    /// INVARIANT: Only allocated resources can be used
    pub fn use_resource(&mut self) {
        if !self.allocated {
            panic!("Attempting to use deallocated resource!");
        }
        println!("Using resource {}", self.id);
    }

    /// Explicitly release the resource
    ///
    /// ORDERING INVARIANT: release() must be called before drop
    pub fn release(mut self) {
        if self.allocated {
            println!("Releasing resource {}", self.id);
            self.allocated = false;
        }
        // self is dropped here, but allocated is now false
    }
}

impl Drop for Resource {
    /// Ensure the resource was properly released
    ///
    /// INVARIANT: Resources should be explicitly released before drop
    fn drop(&mut self) {
        if self.allocated {
            eprintln!("WARNING: Resource {} dropped without being released!", self.id);
            // Emergency cleanup
            println!("Emergency cleanup for resource {}", self.id);
        }
    }
}

/// A scoped resource guard that automatically releases on drop
///
/// INVARIANT: The resource is automatically released when the guard goes out of scope.
/// This is a linear type pattern using RAII.
pub struct ResourceGuard {
    resource: Resource,
}

impl ResourceGuard {
    /// Create a new resource guard
    pub fn new(id: usize) -> Self {
        ResourceGuard {
            resource: Resource::allocate(id),
        }
    }

    /// Access the underlying resource
    pub fn access(&mut self) -> &mut Resource {
        &mut self.resource
    }
}

impl Drop for ResourceGuard {
    fn drop(&mut self) {
        println!("ResourceGuard dropping, ensuring cleanup");
        if self.resource.allocated {
            self.resource.allocated = false;
            println!("Auto-releasing resource {}", self.resource.id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explicit_release() {
        let mut resource = Resource::allocate(1);
        resource.use_resource();
        resource.release();
    }

    #[test]
    fn test_resource_guard() {
        let mut guard = ResourceGuard::new(2);
        guard.access().use_resource();
        // Guard automatically releases on drop
    }
}

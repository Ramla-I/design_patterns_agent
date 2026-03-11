/// Semantic search queries designed to surface latent invariant signals.
/// Each tuple is (query_string, search_mode, description).
pub struct InvariantQuery {
    pub query: &'static str,
    pub mode: &'static str,
    pub description: &'static str,
}

/// Queries targeting different categories of latent invariants.
pub const INVARIANT_QUERIES: &[InvariantQuery] = &[
    // Temporal ordering — "must call X before Y"
    InvariantQuery {
        query: "must call before initialize first",
        mode: "docs",
        description: "Comment-based ordering requirements",
    },
    InvariantQuery {
        query: "not initialized not connected not open error",
        mode: "code",
        description: "Error messages revealing ordering invariants",
    },
    InvariantQuery {
        query: "assert initialized assert connected check state",
        mode: "code",
        description: "Runtime assertions checking state preconditions",
    },

    // State tracking — boolean/enum fields indicating runtime state
    InvariantQuery {
        query: "is_open is_closed is_connected initialized ready",
        mode: "code",
        description: "Boolean fields tracking runtime state",
    },
    InvariantQuery {
        query: "enum State Connecting Connected Closed",
        mode: "code",
        description: "Enums used for runtime state dispatch",
    },

    // Resource lifecycle — acquire/release patterns
    InvariantQuery {
        query: "open close connect disconnect acquire release",
        mode: "code",
        description: "Resource lifecycle method pairs",
    },
    InvariantQuery {
        query: "impl Drop cleanup free deallocate",
        mode: "code",
        description: "Drop implementations and cleanup code",
    },

    // Protocol sequences — multi-step interactions
    InvariantQuery {
        query: "init setup configure start begin",
        mode: "code",
        description: "Initialization and setup sequences",
    },
    InvariantQuery {
        query: "shutdown finalize teardown stop end flush",
        mode: "code",
        description: "Shutdown and finalization sequences",
    },

    // Safety and preconditions
    InvariantQuery {
        query: "SAFETY assumes precondition invariant caller must ensure",
        mode: "docs",
        description: "Safety comments documenting invariants",
    },
    InvariantQuery {
        query: "unsafe is_null ptr valid",
        mode: "code",
        description: "Unsafe code with validity preconditions",
    },

    // Must-use and capability patterns
    InvariantQuery {
        query: "must_use guard token capability permit",
        mode: "code",
        description: "Must-use types and capability tokens",
    },
];

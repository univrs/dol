# Transformations: Processes, Conservation, and Entropy

> "Nothing is lost, nothing is created, everything is transformed."
> — Antoine Lavoisier

## What is a Transformation?

A transformation is an occurrent that changes continuants. It is the bridge 
between states, the mechanism of evolution, the engine of computation.

But transformations are not arbitrary. They are constrained by conservation 
laws, entropy bounds, and causal structure. Understanding these constraints 
is understanding what is possible.

## The Anatomy of Transformation

Every transformation has:

```dol
gene transformation.structure {
  transformation has preconditions
  transformation has postconditions
  transformation has inputs
  transformation has outputs
  transformation has cost
  transformation has duration
  
  preconditions must hold before
  postconditions hold after (if success)
  
  inputs are consumed or read
  outputs are produced or written
  
  cost includes energy and entropy
  duration is bounded below by physics
}

exegesis {
  A transformation is not magic—it has structure. Preconditions
  gate the transformation. Postconditions define success. Inputs
  are resources required. Outputs are results produced. Cost is
  what is spent. Duration is time elapsed.
  
  This structure enables reasoning: Can this transformation occur?
  What will it produce? What will it cost?
}
```

## Conservation Laws

### Conservation of Mass-Energy

In physical systems, total mass-energy is conserved:

```dol
constraint conservation.mass_energy {
  uses transformation.structure
  
  total mass_energy before equals total mass_energy after
  
  mass may convert to energy  // E = mc²
  energy may convert to mass
  
  nothing is created from nothing
  nothing disappears into nothing
}
```

In computing, this manifests as resource accounting:

```dol
constraint conservation.resources {
  uses transformation.structure
  
  memory allocated equals memory freed plus memory in use
  file descriptors opened equals descriptors closed plus descriptors in use
  network connections established equals connections terminated plus connections active
  
  resources do not spontaneously appear
  resources do not spontaneously vanish
}

exegesis {
  Resource leaks violate conservation. When a program slowly
  consumes memory without releasing it, resources accumulate
  without bound. Eventually, the system fails.
  
  Conservation accounting is the foundation of resource management.
}
```

### Conservation of Information (Qualified)

In closed, reversible systems, information is conserved:

```dol
constraint conservation.information_reversible {
  uses transformation.structure
  
  // In reversible transformations
  information before equals information after
  transformation is invertible
  
  no information created
  no information destroyed
}
```

But most computational transformations are irreversible:

```dol
constraint conservation.information_irreversible {
  uses transformation.structure
  
  // In irreversible transformations
  information after is less_than_or_equal information before
  
  information may be lost
  information cannot be created
  
  loss produces entropy
  loss requires minimum energy  // Landauer
}

exegesis {
  When you overwrite a file, the old content is gone. Information
  is lost. This is irreversible—you cannot recover what was not
  preserved elsewhere.
  
  This asymmetry is profound: destruction is easy, creation (of
  information from nothing) is impossible.
}
```

## Entropy and Transformation

### Entropy Production

Every irreversible transformation produces entropy:

```dol
gene transformation.entropy {
  transformation produces entropy
  
  entropy_produced is non_negative
  entropy_produced equals zero only if reversible
  
  entropy flows from transformation to environment
  total entropy never decreases
}

exegesis {
  When a container starts, energy is dissipated as heat.
  When data is written, previous state is erased.
  When logs are truncated, history is lost.
  
  Each of these produces entropy—increases disorder in the
  universe. This is not a flaw but fundamental physics.
}
```

### The Arrow of Transformation

Entropy production gives transformations direction:

```dol
constraint transformation.arrow {
  uses transformation.entropy
  
  transformations have direction
  direction is from lower entropy to higher
  
  reversal requires external work
  spontaneous reversal is forbidden
}

exegesis {
  You can unscramble an egg—but only by expending more energy
  than the scrambling produced. The arrow of transformation
  points toward higher entropy.
  
  This is why undo is expensive, why backups are necessary,
  why recovery is harder than failure.
}
```

## Types of Transformation

### State Transitions

Changes in property values of a continuant:

```dol
gene transformation.state_transition {
  uses transformation.structure
  
  state_transition changes state of continuant
  continuant persists through transition
  identity is preserved
  
  transition has trigger
  trigger may be internal or external
  
  transition has guard
  guard is precondition on current state
  
  transition has action
  action produces postcondition
}

exegesis {
  State transitions are the most common transformations in
  computing. A container goes from "created" to "running".
  A connection goes from "open" to "closed". A file goes
  from "clean" to "dirty".
  
  The key insight: the continuant persists. It's the same
  container, just in a different state.
}
```

### Creations and Destructions

Bringing continuants into or out of existence:

```dol
gene transformation.creation {
  uses transformation.structure
  
  creation brings continuant into existence
  
  creation has substrate  // What it's made from
  creation has template   // What determines its form
  creation has cause      // What initiates creation
  
  before creation: continuant does not exist
  after creation: continuant exists
  
  creation assigns identity
  identity is unique
}

gene transformation.destruction {
  uses transformation.structure
  
  destruction removes continuant from existence
  
  destruction has cause
  destruction may have successor  // What takes its place
  
  before destruction: continuant exists
  after destruction: continuant does not exist
  
  identity is released
  resources are reclaimed
}

exegesis {
  Creation and destruction bracket the lifecycle of every
  continuant. A container is created, exists for a while,
  and is destroyed. Unlike state transitions, creation
  and destruction change what exists, not just how it is.
}
```

### Compositions and Decompositions

Building wholes from parts and vice versa:

```dol
gene transformation.composition {
  uses transformation.structure
  
  composition combines parts into whole
  
  parts exist before composition
  whole exists after composition
  
  parts may cease independent existence
  parts may retain independent existence
  
  whole has emergent properties
  emergent properties not present in parts alone
}

gene transformation.decomposition {
  uses transformation.structure
  
  decomposition separates whole into parts
  
  whole exists before decomposition
  parts exist after decomposition
  
  whole ceases to exist
  parts gain independent existence
}

exegesis {
  A cluster is composed of nodes. A pod is composed of containers.
  A message is composed of bytes. Composition creates wholes with
  properties their parts lack.
  
  When a cluster is decomposed (nodes disconnected), the cluster
  ceases to exist, though the nodes persist.
}
```

### Transfers and Flows

Movement of continuants or properties between contexts:

```dol
gene transformation.transfer {
  uses transformation.structure
  
  transfer moves something from source to target
  
  source has thing before transfer
  target has thing after transfer
  
  transfer has path
  path is sequence of intermediaries
  
  transfer has duration
  duration is bounded by distance over speed
}

exegesis {
  Container migration transfers a container from one node to
  another. Message passing transfers data from sender to receiver.
  The thing transferred may be physical (bytes on wire) or
  logical (ownership of a resource).
}
```

## Transformation Composition

Simple transformations compose into complex processes:

### Sequential Composition

```dol
gene transformation.sequential {
  transformation A followed_by transformation B
  
  postconditions of A enable preconditions of B
  outputs of A may be inputs of B
  
  duration is sum of durations
  cost is sum of costs
  
  failure of A prevents B
}
```

### Parallel Composition

```dol
gene transformation.parallel {
  transformation A concurrent_with transformation B
  
  A and B share no conflicting resources
  A and B may share read-only resources
  
  duration is max of durations
  cost is sum of costs
  
  failure of one may or may not affect other
}
```

### Conditional Composition

```dol
gene transformation.conditional {
  if condition then transformation A else transformation B
  
  condition evaluated before either
  exactly one transformation executes
  
  duration is duration of chosen branch plus evaluation
  cost is cost of chosen branch plus evaluation
}
```

### Iterative Composition

```dol
gene transformation.iteration {
  while condition do transformation A
  
  condition evaluated before each iteration
  transformation executes zero or more times
  
  termination not guaranteed
  termination may be proven for some conditions
}
```

## Transformation Constraints

### Atomicity

```dol
constraint transformation.atomicity {
  atomic transformation completes or does not occur
  
  no partial completion
  no observable intermediate states
  
  failure returns to precondition
  success achieves postcondition
  
  atomicity has scope
  scope defines observability boundary
}

exegesis {
  Atomicity is relative to an observer. A database transaction
  is atomic to clients but not to the disk controller. Defining
  the scope of atomicity is crucial for correctness reasoning.
}
```

### Isolation

```dol
constraint transformation.isolation {
  isolated transformation appears alone
  
  concurrent transformations do not interfere
  intermediate states are not visible to others
  
  isolation has level
  levels trade strictness for concurrency
}
```

### Durability

```dol
constraint transformation.durability {
  durable transformation persists despite failure
  
  effects survive power loss
  effects survive crashes
  effects survive restarts
  
  durability has scope
  scope defines failure modes covered
}
```

## The Cost of Transformation

Every transformation has costs:

```dol
gene transformation.cost {
  cost has energy component
  cost has time component
  cost has entropy component
  cost has opportunity component
  
  energy component is joules expended
  time component is duration
  entropy component is disorder produced
  opportunity component is alternatives foregone
  
  no transformation is free
  minimum costs are bounded by physics
}

exegesis {
  Even "doing nothing" has cost—it takes time, during which
  resources are occupied, energy is consumed maintaining state,
  and opportunities may be missed.
  
  Understanding transformation costs enables optimization:
  choosing the transformation that achieves goals with
  acceptable cost.
}
```

## Transformation in Univrs

The Univrs platform is built on transformations:

```dol
trait univrs.container.operations {
  uses transformation.creation
  uses transformation.destruction
  uses transformation.state_transition
  uses transformation.transfer
  
  create_container is creation of container
  destroy_container is destruction of container
  start_container is state_transition of container
  stop_container is state_transition of container
  migrate_container is transfer of container
  
  each operation has cost
  each operation has duration
  each operation may fail
}
```

```dol
constraint univrs.transformation.integrity {
  uses univrs.container.operations
  
  all transformations preserve cluster invariants
  all transformations are logged
  all transformations are authenticated
  
  failed transformations leave system in valid state
  concurrent transformations do not corrupt state
}
```

---

> "Change is the only constant."
> — Heraclitus

Transformations are how systems evolve. By understanding their structure, 
constraints, and costs, we can design systems that change gracefully rather 
than chaotically.

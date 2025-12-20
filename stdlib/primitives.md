# Primitives: Continuants, Occurrents, and Relations

> "The fundamental concept in social science is Power, in the same sense in 
> which Energy is the fundamental concept in physics."
> — Bertrand Russell

We need a similar fundamental vocabulary for system design.

## The Basic Question

When we design systems, we must answer: **What kinds of things exist?**

This is not a philosophical indulgence—it determines what we can express, 
what we can reason about, and what we can verify.

Drawing from formal ontology (BFO, DOLCE, SUMO) and adapting for system design, 
we identify three primitive categories:

1. **Continuants**: Things that persist through time
2. **Occurrents**: Things that happen in time
3. **Relations**: How things connect to each other

Everything in a system design can be understood in terms of these primitives.

## Continuants: Things That Persist

A continuant is an entity that:
- Exists at any moment it exists
- Maintains identity through change
- Has no temporal parts (it's wholly present at each moment)

### Examples in Computing

| Continuant | Persistence | Identity |
|------------|-------------|----------|
| Container | Exists from creation to removal | Container ID |
| File | Exists from creation to deletion | Inode/path |
| Process | Exists from fork to exit | PID |
| Node | Exists from boot to shutdown | Node ID |
| User | Exists from registration to deletion | User ID |
| Key | Exists from generation to revocation | Key fingerprint |

### Essential Properties of Continuants

```dol
gene continuant.entity {
  entity has identity
  entity has boundaries
  entity has properties
  entity has lifecycle
  
  identity persists through change
  identity is unique within scope
  
  boundaries define what is inside
  boundaries separate from environment
  
  properties may change over time
  some properties are essential
  some properties are accidental
}

exegesis {
  A continuant is the fundamental "thing" in our ontology. Containers,
  nodes, identities, resources—all are continuants. They persist, 
  maintain identity, and can change while remaining themselves.
  
  The distinction between essential and accidental properties is
  crucial: a container can change its resource limits (accidental)
  but not its identity (essential).
}
```

### Identity: The Core of Persistence

Identity is what makes a continuant *that* continuant rather than another.

```dol
gene continuant.identity {
  identity has basis
  identity has scope
  identity has lifetime
  
  basis determines sameness
  scope defines uniqueness domain
  lifetime bounds validity
  
  identity never changes
  loss of identity is destruction
}

exegesis {
  Identity is the anchor of persistence. In Univrs, identity is
  cryptographic (Ed25519 keypairs), ensuring self-sovereignty
  and verifiability without central authority.
  
  The scope matters: a container ID is unique within a cluster,
  a public key is globally unique.
}
```

### Boundaries: The Definition of Self

```dol
gene continuant.boundary {
  boundary separates inside from outside
  boundary defines interaction surface
  boundary may be permeable
  
  permeability is selective
  permeability is controlled
}

exegesis {
  Boundaries are not just physical (container namespaces, network
  segments) but also logical (API surfaces, permission scopes).
  
  A well-designed boundary enables necessary interaction while
  preventing unwanted interference.
}
```

## Occurrents: Things That Happen

An occurrent is an entity that:
- Unfolds in time
- Has temporal parts (beginning, middle, end)
- Does not persist wholly at any moment

### Examples in Computing

| Occurrent | Duration | Participants |
|-----------|----------|--------------|
| Request | Milliseconds to seconds | Client, server |
| Transaction | Variable | Resources, coordinator |
| Deployment | Minutes to hours | Containers, nodes |
| Migration | Seconds to minutes | Container, source, target |
| Consensus round | Milliseconds | Cluster nodes |
| Garbage collection | Milliseconds | Heap, allocator |

### Essential Properties of Occurrents

```dol
gene occurrent.event {
  event has beginning
  event has ending
  event has duration
  event has participants
  
  beginning precedes ending
  duration equals ending minus beginning
  
  participants are continuants
  participation has role
}

exegesis {
  Events are the "happenings" in our ontology. Container starts,
  messages sent, state changes—all are events. Unlike continuants,
  events unfold over time and cannot exist at a single instant.
  
  The participants of an event are the continuants involved.
  A "container start" event involves the container, the runtime,
  and the node.
}
```

### Processes: Extended Occurrents

```dol
gene occurrent.process {
  uses occurrent.event
  
  process has phases
  process has state
  process has goal
  
  phases are ordered
  state tracks progress
  goal defines completion
  
  process may succeed
  process may fail
  process may be interrupted
}

exegesis {
  Processes are complex events with internal structure. A deployment
  is a process with phases (pull image, create container, start,
  health check). It has state (pending, running, failed) and a
  goal (running container).
  
  The possibility of failure and interruption is essential—
  processes in the real world are not guaranteed to complete.
}
```

### Temporal Relations Between Occurrents

Allen's interval algebra gives us precise vocabulary:

```dol
gene occurrent.temporal_relations {
  event A precedes event B
  event A meets event B
  event A overlaps event B
  event A during event B
  event A starts event B
  event A finishes event B
  event A equals event B
  
  // And their inverses
  event B follows event A
  event B met_by event A
  event B overlapped_by event A
  event B contains event A
  event B started_by event A
  event B finished_by event A
}

exegesis {
  Precise temporal reasoning requires precise temporal vocabulary.
  "Before" is ambiguous—does the first event finish before the
  second starts (precedes), or do they share an instant (meets)?
  
  These relations are crucial for causality analysis and 
  consistency reasoning.
}
```

## Relations: How Things Connect

Relations are the connections between entities. They are not themselves 
continuants or occurrents—they are the structure of the ontology.

### Types of Relations

```dol
gene relation.structural {
  // Composition
  part_of relates part to whole
  contains relates whole to part
  
  // Dependency
  depends_on relates dependent to dependency
  supports relates dependency to dependent
  
  // Classification
  instance_of relates individual to type
  subtype_of relates specific to general
}

exegesis {
  Structural relations define how entities compose into larger
  wholes and depend on each other. A container is part_of a pod;
  a pod depends_on a node; a node is instance_of a NodeType.
  
  These relations are persistent—they hold as long as the
  related entities exist in the relationship.
}
```

### Properties of Relations

```dol
gene relation.properties {
  relation has cardinality
  relation has directionality
  relation has transitivity
  
  cardinality constrains count
  // one-to-one, one-to-many, many-to-many
  
  directionality indicates symmetry
  // symmetric: A rel B implies B rel A
  // asymmetric: A rel B implies not(B rel A)
  // non-symmetric: neither
  
  transitivity indicates propagation
  // transitive: A rel B and B rel C implies A rel C
  // intransitive: A rel B and B rel C implies not(A rel C)
  // non-transitive: neither
}

exegesis {
  Understanding relation properties enables reasoning. If
  "depends_on" is transitive, and A depends_on B and B 
  depends_on C, then A depends_on C. This has profound
  implications for failure propagation and startup ordering.
}
```

### Participation Relations

Connecting continuants to occurrents:

```dol
gene relation.participation {
  participation relates continuant to occurrent
  participation has role
  
  role defines nature of involvement
  // agent: causes the event
  // patient: affected by the event
  // instrument: used in the event
  // location: where event occurs
  
  participation has temporal extent
  // throughout: participates for entire duration
  // initially: participates at beginning
  // finally: participates at end
  // intermittently: participates during parts
}

exegesis {
  Participation relations connect the "things" to the "happenings."
  When a container restarts, the container is the patient (affected),
  the orchestrator is the agent (causes), and the node is the
  location (where it happens).
}
```

## Composing Primitives

These primitives compose to describe complex systems:

```dol
gene container.exists {
  // Container is a continuant
  container is continuant
  
  container has identity     // From continuant.identity
  container has boundaries   // From continuant.boundary
  container has state        // Accidental property
  container has resources    // Accidental property
}

trait container.lifecycle {
  // Lifecycle events are occurrents
  uses container.exists
  
  creation is event where container is patient
  start is event where container is patient
  stop is event where container is patient
  removal is event where container is patient
  
  creation precedes start
  start precedes stop
  stop precedes removal
  
  each event has agent    // Who caused it
  each event has timestamp // When it happened
}

constraint container.identity_persistence {
  uses container.exists
  uses container.lifecycle
  
  container identity never changes
  // Across all lifecycle events
  
  identity at creation equals identity at removal
}
```

## The Primitive Hierarchy

```
                    ┌─────────────┐
                    │   Entity    │
                    └──────┬──────┘
                           │
           ┌───────────────┼───────────────┐
           │               │               │
    ┌──────▼──────┐ ┌──────▼──────┐ ┌──────▼──────┐
    │ Continuant  │ │  Occurrent  │ │  Relation   │
    └──────┬──────┘ └──────┬──────┘ └──────┬──────┘
           │               │               │
    ┌──────┴──────┐ ┌──────┴──────┐ ┌──────┴──────┐
    │  Physical   │ │   Event     │ │ Structural  │
    │  Abstract   │ │   Process   │ │ Temporal    │
    │  Information│ │   History   │ │ Participatory│
    └─────────────┘ └─────────────┘ └─────────────┘
```

## Why These Primitives?

These categories are not arbitrary. They reflect deep structure in how we 
understand reality:

1. **Continuants** capture what *persists*—the things we name, manage, and 
   care about over time.

2. **Occurrents** capture what *happens*—the changes, actions, and events 
   that matter to us.

3. **Relations** capture how things *connect*—the structure that makes 
   isolated entities into coherent systems.

Together, they provide a complete vocabulary for describing any system at 
any level of abstraction.

## Connecting to Other Layers

**Downward (to Foundations)**:
- Continuants have physical substrate (matter, energy)
- Occurrents obey causality and thermodynamics
- Relations are constrained by physical possibility

**Upward (to Transformations)**:
- Transformations are special occurrents that change continuants
- Conservation laws constrain what transformations are possible
- Information flow is a type of relation with special properties

---

> "The purpose of abstraction is not to be vague, but to create a new 
> semantic level in which one can be absolutely precise."
> — Edsger Dijkstra

These primitives are precise enough to reason about, general enough to 
apply universally, and grounded enough to connect to physical reality.

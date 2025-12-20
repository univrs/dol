# Foundations: Physics, Causality, and Information

> "Nature uses only the longest threads to weave her patterns, so that each 
> small piece of her fabric reveals the organization of the entire tapestry."
> — Richard Feynman

## The Physical Substrate of Computation

Every computation, every data structure, every algorithm exists as a pattern 
in physical matter. There is no ethereal realm where software floats free of 
hardware. This is not a limitation—it is the ground truth that makes our 
abstractions meaningful.

Understanding the physical foundations is not optional for serious system 
design. It tells us what is possible, what is efficient, and what is fundamentally 
impossible.

## The Laws of Thermodynamics

### The First Law: Conservation of Energy

Energy cannot be created or destroyed, only transformed.

**Implications for computing:**
- Every computation requires energy input
- That energy must come from somewhere (power supply, battery, solar)
- The energy is transformed, not consumed (becomes heat, electromagnetic radiation)

```dol
constraint thermodynamics.first_law {
  all computation requires energy
  energy input equals energy output
  no energy is created
  no energy is destroyed
}
```

### The Second Law: Entropy Increases

In any closed system, entropy (disorder) never decreases.

**Implications for computing:**
- Systems naturally drift toward disorder
- Maintaining order requires continuous energy expenditure
- Error correction is fighting entropy
- Backup and redundancy are entropy insurance

```dol
constraint thermodynamics.second_law {
  entropy does not decrease spontaneously
  order requires energy to maintain
  all physical systems tend toward equilibrium
  
  information degrades over time
  redundancy fights degradation
  maintenance is not optional
}
```

This is why:
- Disks fail and data corrupts
- Networks drop packets
- Memory bits flip
- Heat sinks are necessary

We are not fighting bugs—we are fighting the universe's fundamental tendency 
toward disorder.

### The Third Law: Absolute Zero is Unreachable

As temperature approaches absolute zero, entropy approaches a minimum.

**Implications for computing:**
- Perfect error-free computation is impossible at non-zero temperature
- Quantum computing requires extreme cooling precisely because of this
- Noise is fundamental, not incidental

### Landauer's Principle

The minimum energy to erase one bit of information is:

```
E = kT ln(2)
```

Where:
- k = Boltzmann's constant
- T = temperature in Kelvin

At room temperature (~300K), this is approximately 3 × 10⁻²¹ joules per bit.

**Implications:**
- Information has physical reality
- Forgetting has a cost
- Reversible computation is theoretically more efficient
- There is a fundamental connection between information and thermodynamics

```dol
constraint landauer.principle {
  erasing information requires energy
  minimum energy equals kT times ln(2) per bit
  
  irreversible operations waste energy
  reversible operations preserve energy
  
  garbage collection has thermodynamic cost
}
```

## Causality

### The Arrow of Time

Effects follow causes. The future depends on the past. Time flows in one direction.

This seems obvious, but its implications are profound:

- **Ordering matters**: Operations cannot be arbitrarily reordered
- **History is real**: The past constrains the present
- **Prediction is limited**: Chaos and quantum mechanics bound knowability

```dol
constraint causality.temporal {
  effects follow causes
  causes precede effects
  simultaneity is relative  // Special relativity
  
  no effect without cause
  no backward causation
}
```

### Finite Speed of Light

Information cannot travel faster than c ≈ 3 × 10⁸ m/s.

In a datacenter:
- Light travels ~30cm per nanosecond
- Cross-continent latency is physically bounded (~50ms minimum US coast-to-coast)
- "Instant" global consistency is impossible

```dol
constraint causality.speed_limit {
  information propagates at finite speed
  maximum speed is c
  
  latency has physical minimum
  simultaneity requires definition
  
  consensus requires communication
  communication requires time
}
```

**Implications for distributed systems:**
- CAP theorem is not a bug—it's physics
- Strong consistency across distance has minimum latency
- Eventually consistent systems respect physics more honestly

### Locality

Physical effects are local. Action at a distance requires a mediating field or particle.

**Implications:**
- Remote procedure calls are not like local calls
- Network partitions are possible
- State must be explicitly synchronized

## Information Theory

### Shannon's Entropy

Information content of a message is measured by surprise:

```
H(X) = -Σ p(x) log₂ p(x)
```

**Key insights:**
- Predictable messages carry little information
- Random messages carry maximum information
- Compression cannot exceed entropy bounds

```dol
gene information.entropy {
  information has entropy
  entropy measures uncertainty
  
  predictable has low entropy
  random has high entropy
  
  compression is bounded by entropy
  no lossless compression below entropy
}
```

### Channel Capacity

Every communication channel has a maximum rate at which information can be 
reliably transmitted:

```
C = B log₂(1 + S/N)
```

Where:
- B = bandwidth
- S = signal power
- N = noise power

**Implications:**
- Bandwidth is not the only limit
- Noise fundamentally constrains communication
- Error correction trades rate for reliability

```dol
constraint channel.capacity {
  channels have maximum capacity
  capacity depends on bandwidth and noise
  
  reliability trades against throughput
  error correction reduces effective rate
  
  no infinite bandwidth
  no zero noise
}
```

### Data Processing Inequality

Processing information cannot increase its content:

```
I(X; Y) ≥ I(X; f(Y))
```

For any function f, processing Y cannot give you more information about X.

**Implications:**
- Derived data is never more informative than source data
- Summarization loses information
- Machine learning cannot create information, only extract patterns

```dol
constraint information.processing {
  processing cannot increase information
  summarization loses detail
  inference cannot exceed evidence
  
  garbage in, garbage out
  signal cannot emerge from pure noise
}
```

## The Physical Ontology

Grounding these principles in DOL:

```dol
gene physical.system {
  system has energy
  system has entropy
  system has state
  system has boundaries
  
  state changes require energy
  entropy tends to increase
  boundaries define inside from outside
}

exegesis {
  Every computational system is a physical system. The electrons
  flowing through circuits, the photons in fiber optics, the
  magnetic domains on disk platters—these are the physical reality
  underlying all our abstractions.
  
  This gene captures the minimal physical properties that any
  honest model of computation must acknowledge.
}
```

```dol
trait physical.evolution {
  uses physical.system
  
  system evolves over time
  evolution follows physical law
  
  each state transition consumes energy
  each state transition may increase entropy
  each transition respects causality
}

exegesis {
  Systems change. They boot, run, fail, recover, degrade, and
  eventually terminate. This evolution is not arbitrary—it
  follows physical law. Energy is consumed, entropy increases,
  causes precede effects.
}
```

```dol
constraint physical.honesty {
  uses physical.system
  uses physical.evolution
  
  no perpetual motion
  no maxwell demons
  no faster than light
  no negative entropy spontaneously
  
  all abstractions have physical cost
  all operations have minimum latency
  all storage has minimum energy
}

exegesis {
  Honesty about physics is the foundation of reliable systems.
  We cannot abstract away thermodynamics. We cannot ignore
  the speed of light. We cannot pretend that operations are
  free.
  
  This constraint codifies the non-negotiable realities that
  every system must respect.
}
```

## Why This Matters

These foundations matter because they define **what is possible**.

| Fantasy | Reality | Implication |
|---------|---------|-------------|
| Infinite bandwidth | Bounded by physics | Design for limited throughput |
| Zero latency | c is finite | Design for delay |
| Perfect reliability | Entropy increases | Design for failure |
| Free computation | Energy required | Design for efficiency |
| Permanent storage | Degradation inevitable | Design for redundancy |

Systems that ignore these realities fail. Systems that respect them are robust.

## Connecting to Higher Layers

The foundations constrain but do not determine. Within physical law, vast 
design space remains.

The next documents explore:

- **Primitives**: The abstract building blocks (entities, events, relations)
- **Transformations**: How things change while respecting constraints
- **Information**: The bridge between physical and abstract
- **Application**: How Univrs instantiates these principles

Each layer is consistent with the foundations but adds structure appropriate 
to its level of abstraction.

---

> "The universe is not only queerer than we suppose, but queerer than we 
> *can* suppose." — J.B.S. Haldane

Our ontology need not capture all of physics—only enough to keep our 
abstractions honest and our systems robust.

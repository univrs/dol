# Information: Bits, Atoms, Encoding, and Fidelity

> "Information is physical."
> — Rolf Landauer

## The Nature of Information

Information is not abstract. It is not ethereal. It is not separate from 
the physical world. Information is always embodied in physical substrate—
patterns of matter and energy that can be read, written, transmitted, and 
transformed.

This physical grounding is not incidental but **fundamental** to understanding 
what information systems can and cannot do.

## The Bit: Fundamental Unit

A bit is a distinction between two states. Nothing more, nothing less.

```dol
gene information.bit {
  bit has two states
  states are distinguishable
  states are stable (within tolerance)
  
  state can be read
  state can be written
  state can be transmitted
  
  reading requires energy
  writing requires energy
  transmitting requires energy
}

exegesis {
  The bit is the atom of information. Whether encoded as voltage
  levels in silicon, magnetic domains on disk, photon polarization
  in fiber, or any other physical distinction, a bit is a choice
  between two alternatives.
  
  The physical realization varies; the logical structure persists.
}
```

## Physical Realizations

Information requires physical substrate:

### Electronic

```dol
gene information.substrate.electronic {
  uses information.bit
  
  bit encoded as voltage level
  high voltage represents one
  low voltage represents zero
  
  switching requires energy
  switching has delay
  switching produces heat
  
  susceptible to electrical noise
  susceptible to cosmic rays
  susceptible to thermal fluctuation
}
```

### Magnetic

```dol
gene information.substrate.magnetic {
  uses information.bit
  
  bit encoded as magnetic orientation
  north represents one
  south represents zero
  
  writing requires magnetic field
  reading requires sensing field
  
  persistent without power
  susceptible to magnetic interference
  susceptible to physical shock
  degrades over time
}
```

### Optical

```dol
gene information.substrate.optical {
  uses information.bit
  
  bit encoded as light property
  // intensity, polarization, wavelength, phase
  
  transmission at light speed
  low interference between channels
  
  requires line of sight or waveguide
  susceptible to physical obstruction
  requires conversion at endpoints
}
```

### Quantum

```dol
gene information.substrate.quantum {
  uses information.bit
  
  qubit has superposition of states
  measurement collapses superposition
  entanglement enables non-local correlation
  
  enables new computational primitives
  requires extreme isolation
  susceptible to decoherence
  error correction is expensive
}

exegesis {
  Quantum information is not just "faster bits" but a fundamentally
  different computational model. Our ontology must be flexible enough
  to accommodate this while remaining grounded in physical reality.
}
```

### Biosynthetic (Future)

```dol
gene information.substrate.biosynthetic {
  uses information.bit
  
  bit encoded in molecular state
  // DNA base pairs, protein conformations
  
  massive parallelism possible
  self-replication possible
  extremely dense storage
  
  slow switching
  requires wet environment
  susceptible to biological degradation
}

exegesis {
  Biology has been computing for billions of years. As we learn
  to engineer biological systems, they become viable information
  substrates. The ontology remains valid—only the physical
  realization changes.
}
```

## Encoding: Symbols to States

Encoding is the mapping from abstract symbols to physical states:

```dol
gene information.encoding {
  encoding maps symbols to states
  decoding maps states to symbols
  
  encoding has alphabet
  alphabet is set of symbols
  
  encoding has codewords
  codewords are sequences of states
  
  encoding may be fixed length
  encoding may be variable length
  
  decoding must be unambiguous
  // Every codeword maps to exactly one symbol
}

exegesis {
  ASCII encodes 128 characters as 7-bit patterns. UTF-8 encodes
  millions of codepoints as variable-length byte sequences.
  Protocol Buffers encode structured data as binary streams.
  
  The encoding is not the information—it is how information
  is represented in a particular substrate.
}
```

### Properties of Encodings

```dol
gene encoding.properties {
  uses information.encoding
  
  // Efficiency
  encoding has redundancy
  redundancy is overhead beyond minimum
  
  // Error detection
  encoding may detect errors
  detection requires redundancy
  
  // Error correction
  encoding may correct errors
  correction requires more redundancy
  
  // Compression
  encoding may compress
  compression reduces redundancy
  compression has limits (entropy bound)
}
```

## The Channel: Information in Transit

```dol
gene information.channel {
  channel connects source to destination
  
  channel has capacity
  capacity is maximum bits per second
  
  channel has noise
  noise corrupts information
  
  channel has latency
  latency is minimum transit time
  
  channel has bandwidth
  bandwidth constrains capacity
}

exegesis {
  Every communication is through a channel. Network links, shared
  memory, message queues, even function calls—all are channels
  with capacity, noise, and latency characteristics.
}
```

### Shannon's Channel Capacity

```dol
constraint channel.shannon {
  uses information.channel
  
  maximum reliable rate is bounded
  bound is B * log2(1 + S/N)
  
  B is bandwidth
  S is signal power
  N is noise power
  
  exceeding capacity causes errors
  errors require retransmission or loss
}

exegesis {
  Shannon's theorem is profound: there is a maximum rate at which
  information can be reliably transmitted, and this rate depends
  on physical properties of the channel.
  
  No clever encoding can exceed this limit. No protocol can violate
  Shannon. This is physics, not engineering.
}
```

## Fidelity: Information Preservation

Fidelity measures how well information is preserved through transformation:

```dol
gene information.fidelity {
  fidelity measures preservation
  
  perfect fidelity: output equals input
  lossy fidelity: output approximates input
  zero fidelity: output independent of input
  
  fidelity may degrade through transformations
  fidelity never increases (data processing inequality)
}

exegesis {
  Every copy, transmission, transformation, and storage operation
  has fidelity characteristics. Lossless compression has perfect
  fidelity. JPEG has lossy fidelity. A hash has zero fidelity
  for recovering input (one-way function).
}
```

### Sources of Fidelity Loss

```dol
gene fidelity.degradation {
  uses information.fidelity
  
  noise corrupts information
  quantization loses precision
  sampling loses continuity
  compression loses detail
  abstraction loses specificity
  
  each transformation may reduce fidelity
  no transformation can increase fidelity
  
  redundancy protects against corruption
  redundancy cannot prevent loss
}
```

## Bits ↔ Atoms: The Bidirectional Bridge

Univrs operates at the interface between digital and physical:

### Actuation: Bits → Atoms

```dol
trait information.actuation {
  uses information.encoding
  uses transformation.structure
  
  digital state causes physical action
  
  actuation has latency
  actuation has energy cost
  actuation has success probability
  
  action may differ from intent
  difference is error
  error may be detectable
  error may be correctable
}

exegesis {
  When a container starts, digital state (container spec) causes
  physical action (process execution). This is actuation—information
  becoming physical effect.
  
  Actuation is never perfect. The process might fail to start.
  The resources might be unavailable. The action might timeout.
}
```

### Sensing: Atoms → Bits

```dol
trait information.sensing {
  uses information.encoding
  uses transformation.structure
  
  physical state becomes digital representation
  
  sensing has precision
  precision bounds accuracy
  
  sensing has sampling rate
  rate bounds temporal resolution
  
  sensing loses information
  not all physical detail is captured
  
  sensing has noise
  noise corrupts representation
}

exegesis {
  When a metric is collected, physical state (CPU temperature,
  memory usage, network bytes) becomes digital representation
  (integers, floats, timestamps). This is sensing—physical
  reality becoming information.
  
  Sensing is never perfect. The metric has limited precision.
  The sampling might miss events. The sensor might have noise.
}
```

### The Sensing-Actuation Loop

```dol
trait information.control_loop {
  uses information.sensing
  uses information.actuation
  
  sense current state
  compare to desired state
  compute action to reduce difference
  actuate the computed action
  repeat
  
  loop has period
  loop has stability
  loop may converge or diverge
  
  disturbances perturb loop
  controller compensates for disturbances
}

exegesis {
  Feedback control is the fundamental pattern for systems that
  interact with physical reality. The Kubernetes controller
  pattern, the thermostat, the cruise control—all are instances
  of this loop.
  
  Univrs orchestration is a control loop: sense cluster state,
  compare to declared intent, compute reconciliation actions,
  actuate the changes.
}
```

## Information Lifecycle

```dol
gene information.lifecycle {
  information is created
  information is stored
  information is transmitted
  information is processed
  information is destroyed
  
  each stage has fidelity characteristics
  each stage has cost
  each stage has duration
  
  total fidelity is bounded by worst stage
}
```

### Creation

```dol
gene information.creation {
  uses information.lifecycle
  
  creation may be sensing (physical to digital)
  creation may be computation (from other information)
  creation may be random (from entropy source)
  
  creation requires energy
  creation defines initial state
}
```

### Storage

```dol
gene information.storage {
  uses information.lifecycle
  
  storage preserves information over time
  
  storage has capacity
  storage has latency
  storage has bandwidth
  storage has retention
  
  retention is time before degradation
  degradation is fidelity loss
  redundancy extends retention
}
```

### Transmission

```dol
gene information.transmission {
  uses information.lifecycle
  uses information.channel
  
  transmission moves information through space
  
  transmission has latency
  transmission has bandwidth
  transmission has reliability
  
  reliability is inverse of error rate
  error rate depends on channel
  error correction trades bandwidth for reliability
}
```

### Processing

```dol
gene information.processing {
  uses information.lifecycle
  uses transformation.structure
  
  processing transforms information
  
  processing has latency
  processing has throughput
  processing has energy cost
  
  processing cannot increase information content
  processing can extract patterns
  processing can reduce volume
}
```

### Destruction

```dol
gene information.destruction {
  uses information.lifecycle
  
  destruction removes information from existence
  
  destruction has minimum energy cost  // Landauer
  destruction is often irreversible
  destruction may leave traces (forensics)
  
  secure destruction requires overwriting
  cryptographic destruction requires key destruction
}
```

## Information in Univrs

```dol
system univrs.information @ 0.1.0 {
  requires information.encoding >= 0.0.1
  requires information.channel >= 0.0.1
  requires information.fidelity >= 0.0.1
  
  cluster state is information
  event stream is information channel
  logs are information storage
  metrics are sensing results
  commands are actuation requests
  
  all information has encoding
  all transmission has fidelity bounds
  all storage has retention limits
}

exegesis {
  Univrs is, fundamentally, an information system. It senses
  the state of the cluster, processes this information to
  determine actions, actuates changes in physical resources,
  and records the results.
  
  Understanding information properties—capacity, fidelity,
  latency—is essential for understanding Univrs behavior.
}
```

## The Information-Theoretic View

From information theory's perspective, a computing system is:

1. **Sources**: Generate information (sensors, inputs, random events)
2. **Channels**: Transmit information (networks, buses, queues)
3. **Processors**: Transform information (CPUs, algorithms, functions)
4. **Sinks**: Consume information (actuators, displays, storage)

The entire system obeys information-theoretic laws:
- Capacity bounds
- Data processing inequality
- Entropy constraints
- Fidelity limits

These are not implementation details—they are **fundamental constraints** 
that any system must respect.

---

> "The measure of information is surprise."
> — Claude Shannon

Information is what reduces uncertainty. And information is physical—always 
embodied, always constrained, always precious.

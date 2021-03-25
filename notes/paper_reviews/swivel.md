# Swivel
### Abstract + Methodology
This [paper](https://arxiv.org/pdf/2102.12730.pdf) proposes a compiler framework which provides strong in-process/in-memory isolation and hardening of spectre attacks against WebAssembly (Wasm). It provides two solutions, with probabilistic (ASLR + flush) or deterministic (rewrite?)
1. Swivel Software Fault Isolation - a software based mechanism.
   - Linear blocks
     - Control flow transfer at Terminators of blocks.
     - memory access within block is masked to sandbox memory.
   - No `ret` instructions.
   - Use a `shadow stack` - separate stack for return addresses.
   - *Probabilistic*: ASLR of Wasm Sandbox + Branch Target Buffer(BTB) flush.
   - *Deterministic*: convert conditional branches to indirect branches + BTB flush.
2. Swivel Control-flow Enforcement Technology - a hardware based mechanishm.
   - Intel Control-flow Enforcement Technology(**CET**) + Intel Memory Protection Keys(**MPK**)
   - Use of Linear blocks
   - Use Intel CET's `shadow stack` and don't bother removing `ret`.
   - Strong isolation between Host and Sandboxes using Intel MPK.
   - Use `endbranch` instructions as valid jump targets.
   - *Probabilistic*: ASLR of Wasm Sandbox + BTB flush.
   - *Deterministic*: `register interlock` converts misspeculated memory access to guard page.
     - introduces a data dependency between memory ops and resolution of control flow.
   - No need to flush BTB.

### Implementation + Evaluation
1. Swivel extends `Lucet` and its `Cranelift` code as compiler passes.
   - Minimal insertion of pipeline fences.
   - No annotation needed. / works transparently.
   - Modular / Configurable.
2. Linear blocks.
    - Masking memory accesses: replace `Craenelift`'s `mask-before-spill` approach with `mask-after-unspill` approach.
    - Pinning the heap registers: Reserves `pinned heap register` to store the address of the sandbox heap.
    - Hardening jump tables: bounds check using `speculative load hardening`.
    - Protecting returns: use `separate stack` or `shadow stack`.
3. Extends Google's `Safeside` suite to perform attacks.

### Results / Conclusion
Swivel eliminates three kinds of Spectre attacks through CET + MPK and mitigates using SFI.
Swivel-SFI has relatively less overhead than Swivel-CET.

### Limitations + Differentiation.
- Swivel assumes that there will not be any secret information on the host.
  - It expects the secrets to be moved to a sandbox. But how? We don't have that information.
- Most of the time, it assumes that Hyperthreading is disabled. May be it is necessary for most of the defenses.
- Doesn't solve Spectre-STL and many others. Didn't explore much though.
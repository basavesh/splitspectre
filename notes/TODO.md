# TODO

- [x] Find a way to do type inference at a function signature level.
- [ ] Automatically transform the original code to split type.
- [ ] Setup the infrastructure automatically such that they will work.
  - [ ] Should we take advantage of Intel Trust Domain Extensions(Intel TDX)
  - [ ] Take a look at Bareflank Hypervisor.
- [ ] Define the Threat / Attacker Model and security guarantees.
- [ ] Evaluation
  - [ ] Performance Overhead
  - [ ] Portability
  - [ ] Security defenses
  - [ ] limitations?


## Theorems
- Functional: The split program is functionally equivalent to the regular program.
- Safety: The split program provides stronger isolation by separating the secret data and secret execution into an another process.
  - Hardening against Spectre attacks:
  - Framework to verify that trusted code runs in speculative constant-time.
  - Verifying the small Trusted Code Base(TCB)?
  - Hardware-assisted or software based
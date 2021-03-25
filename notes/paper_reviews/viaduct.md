# Viaduct
- Looks like an extension to Jiff/Split Program, which adds some compiler extension capability.
- Allows mutually distructing principals to perfrom Multi-Party Compuations.
- Written in high-level language with user annotations(`FLAM Security Labels`) and compiled to MPC backend.
- Source Program -> Label Inference -> Protocol Selection -> (distributed program) -> Runtime.
- Idea: translate information flow typing constraints into authority constraints.
- Solution: solve for minimum authority solution.
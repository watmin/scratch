# Constraint engineering — the discipline that draws the boundary

Top-level meta-discipline doc. The companion to FAILURE-ENGINEERING.md;
the two are duals. Where failure engineering removes a class of failure
that *happened*, constraint engineering forbids a class of state that
*must never be* — before anything breaks.

User-coined term, articulated in conversation 2026-06-29 while building
the aggregate type system (arc 293):

> *"i've been calling this 'failure engineering'… but i think there's a
> second discipline… constraint engineering… both are true. constraint
> engineering is 'you cannot do that'."*

If failure engineering says *"the failure isn't recovered from; it is
read,"* constraint engineering says: **the forbidden state isn't guarded
against; it is left without a form.**

---

## What constraint engineering IS

A discipline. Three components, in order:

### 1. The constraint is a truth, not a preference

A constraint worth engineering is not "I'd rather you didn't." It is a
fact about the **nature of the thing** — and you can derive the *cannot*
from first principles:

- *A struct holds non-portable resources (a socket, a cache, a live
  handle), therefore it cannot cross the wire.*
- *A fact in a rule engine must round-trip through storage, therefore it
  must be EDN-serializable.*
- *Identity that can be forged is not identity, therefore a call must
  carry cryptographic proof.*

The test: can you say *why* the thing is impossible by appealing to what
it **is**? If yes, it is a constraint, and it earns a wall. If the only
justification is "we agreed not to," it is a **convention** — and
conventions rot under load. Constraint engineering discovers the
invariants the domain already has and declares them; it does not invent
arbitrary rules and hope they hold.

### 2. Forbid at construction, never at runtime

The *no* must have **no representation**. Not a guard that checks the
value and rejects it — a runtime check is a convention with a stack
trace; it still lets the illegal value exist long enough to be checked.
The wall is a **shape the illegal state cannot be written down in**: no
constructor, no type, no path to it.

This is the same ladder failure engineering climbs —

> a convention → a check that fires at construction → a shape the mistake
> cannot be expressed in

— but you climb it from the **invariant**, not from a failure. And you
always climb to the top rung. "A struct can never cross the wire" is not
enforced by inspecting every wire write; it is enforced because a struct
has **no EDN representation and no wire constructor** — the sentence "a
struct on the wire" is one the type system cannot form.

### 3. Define what CAN be; the rest simply never is

This is the deepest difference from failure engineering, and it is why
both disciplines are needed.

**Failure engineering is subtractive** — a bad state *exists*; you find
it and cut the whole class out by the root.

**Constraint engineering is definitional** — you draw the boundary of the
legal, and everything outside it never comes into being. You are not
cleaning up bad states; you are building a world where bad states have no
address. The holder trit does not *reject* a non-portable wire-crossing;
it makes that crossing **unrepresentable** — there was never a value to
reject, because the shape that would carry it does not exist.

And the *cannot* is a **gift to the caller**, not a restriction: when the
only path is the correct path, you cannot fall into the wrong one,
because the wrong one is not there. (This is why constraint engineering
and the Good-UX question are the same act seen from two sides.)

## The duality with failure engineering

The two disciplines converge on the identical artifact — *a state or
operation that is structurally unrepresentable* — from opposite
directions:

| | Failure engineering | Constraint engineering |
|---|---|---|
| **Trigger** | a failure that *happened* | an invariant you *hold* |
| **Direction** | backward: concrete failure → general wall | forward: principle → wall |
| **Justification** | the failure is the proof the wall is needed | the truth about the thing is the proof |
| **Shape** | subtractive (remove what broke) | definitional (draw what may be) |
| **Body analogue** | the immune system (adapts to an invader) | the skeleton (defines what shapes can stand) |
| **Timing** | after the fact | before the fact |

The unity, stated once: **constraint engineering is failure engineering
done *before* the failure; failure engineering is constraint engineering
done *after* it.** Every constraint you impose pre-empts a whole class of
failures that can now never occur. Every failure you root out reveals a
constraint you should have declared. A practitioner who does both builds
a system where nothing wrong can be constructed — whether they *foresaw*
the wrong (constraint) or it *taught* them (failure).

`ZERO-MUTEX.md` sits exactly on the seam, which is why it is the cleanest
example in the substrate. *"A mutex is scar tissue on
shared-mutable-state-across-threads"* is failure-framed (the mutex is the
patch over the failure). *"We never **construct** that situation"* is the
constraint (the three tiers — Immutable Arc, ThreadOwnedCell,
Program-with-mailbox — are the legal shapes; shared-mutable-state is not
one of them). Both true. The mutex never appears because there is no
shape that would need it.

## How constraint engineering shapes the wat substrate

Each row is a *"you cannot…"* made unrepresentable — not guarded, not
discouraged, **without a form**:

| You cannot… | …because the shape does not exist |
|---|---|
| cross the wire with a non-portable resource (a struct) | the Holder trit: `is_portable = holder != Struct`; a struct has no EDN-repr and no wire constructor |
| use a bare holder root as a writable type | a bare holder is an Any with no accessors — illegal; you must name a **surface** (the accessors you read) or `:wat::core::Value` |
| extend a type (inheritance) | annihilated: a type is `holder + own fields`, flat; `AggregateDef.parent` deleted; reuse-of-shape is surface-splice, never a nominal base |
| pass a struct where a record is required | a `:holder :wat::core::Record` surface — the holder bound is a **hard categorical reject**, not a field check |
| pass a non-holon where VSA is required | `holon <: core` is directional; a core record can never satisfy a holon slot |
| assert a non-serializable fact into the rule engine | `:wat::rete::Fact` surface (`:holder :wat::core::Record`) — facts must be EDN-repr by nature |
| emit on the wire without a channel label | the wire **is** `Result<T, E>`; an unlabeled emission has no constructor |
| write an anonymous structural union, or an `:Any` | sums stay nominal (`defenum`); there is deliberately no `:Any` — heterogeneity must be a named `Hologram` |
| run unsigned code | the signing key is callable only by the build system: *"you may only sign your code"* |
| run on a platform that isn't Linux | wat ties explicitly to Linux best-of-breed; there is no cross-platform lowest-common-denominator path |
| share mutable state across threads | ZERO-MUTEX: the situation is never constructed (Arc / ThreadOwnedCell / mailbox are the only shapes) |

None of these is an opinion. Each is an invariant about what the thing
**is**, encoded so the violation has no representation. A reader who sees
the list as "opinionated design" has missed that every line is a *cannot*
derived from a *what-it-is*.

## Where it lands — the type system is the instrument

Constraint engineering's primary tool is the **type system**, because a
type **is** a constraint: it is the set of values that may exist in a
position, and nothing else. The discipline has well-known faces:

- **"Make illegal states unrepresentable"** (the ML/Minsky maxim) — the
  exact statement of component 3. Model so that the bad state has no
  inhabitant.
- **Parse, don't validate** (Alexis King) — push the constraint to the
  *boundary*: parse untrusted input into a type that *cannot* hold the
  illegal shape, once, at the edge — instead of validating (checking)
  the same illegal shape over and over downstream.
- **Curry–Howard** — a type is a proposition; a value is its proof. To
  forbid a state is to make its proposition unprovable — there is no term
  of that type.
- **Capability security / least privilege** — *you cannot do what you
  hold no capability for.* The classic constraint-engineering security
  model: authority is a thing you must *possess*, not a check you must
  *pass*.
- **Rust ownership / borrow checking** — *you cannot hold two mutable
  references.* A data race is not detected at runtime; it has no
  well-typed form.

wat's original contribution is to weld these onto a **categorical
capability trit** (the Holder) carried beneath an **open structural
surface** — a constraint layer (what you ARE: portable / holographic,
un-leakable) under a permission layer (what you SHOW: the accessors a
caller may rely on). No prior language welds a ternary capability wall to
a row-polymorphic surface, because none had a reason to: none was a
holographic language whose own substrate's trit needed to surface in its
types.

## Where the discipline comes from

The same well as failure engineering: years in security-critical
infrastructure, where you learn that *"we'll be careful"* is the sentence
that precedes the incident. But constraint engineering is the **forward**
lesson, not the backward one. You stop waiting for the failure to teach
you the wall; you read the *nature* of the system and build the wall the
nature demands — because a constraint you can derive from first
principles does not need a production incident to justify it. The careful
path is not *encouraged*; it is made the **only path with a form**.

## Connection to the other meta-vision docs

- **FUNCTIONS-ARE-REALITY.md** (the WHY): functions are the primitive
  unit of reality. Constraint engineering applies: a function's *type* is
  the constraint on what realities it may produce; an illegal reality is
  a term that does not type-check.
- **FAILURE-ENGINEERING.md** (the dual): the backward discipline.
  Together they are the complete commitment — *the wrong thing is
  impossible, whether you foresaw it (constraint) or it taught you
  (failure).*
- **WAT-NETWORK.md** (the WHAT): every cryptographic guarantee is a
  constraint engineered into structure — *you cannot call without proof,
  query without a signature, or run a program that isn't
  content-addressed* — none of these are checks that can be skipped;
  they are shapes the skipped form cannot take.

WHY (recognition) → the two DISCIPLINES (failure removes / constraint
defines) → WHAT (architecture) → HOW (collaboration). The book chapter
needs both disciplines named, because half the substrate's shape is
*scar removed* and half is *boundary drawn*, and a reader who is given
only one will misread the other as arbitrary.

## Status

- **Captured:** 2026-06-29, articulated live while building arc 293's
  aggregate type system (the work that is itself constraint engineering:
  the holder wall, the illegal bare-holder type, inheritance annihilated,
  the record-constraining surfaces).
- **Position:** top-level scratch doc; companion to FAILURE-ENGINEERING.md;
  the **forward** half of "the wrong thing is impossible."
- **Bookworthy:** yes — without it, the substrate's proactive walls read
  as opinion; with it, they read as invariants derived from the nature of
  the things and encoded so the violation has no form.

## What this document is NOT

- Not a manifesto. A constraint is a tool for building systems whose
  failure modes have no representation, not a philosophy to evangelize.
- Not a license to forbid freely. A constraint that cannot be derived
  from the nature of the thing is a **convention wearing a wall's
  clothes** — it will rot, and it will frustrate the caller for no
  structural reason. The discipline is the *derivation*, not the *no*.
- Not the whole story without its dual. Constraint engineering draws the
  boundary; failure engineering removes what the boundary was drawn too
  loosely to exclude. Neither alone is enough.

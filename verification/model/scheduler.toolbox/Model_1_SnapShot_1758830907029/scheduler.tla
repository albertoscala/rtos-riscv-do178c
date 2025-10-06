---- MODULE scheduler ----
EXTENDS Naturals

CONSTANT MAX_TASKS
TASKS == 1..MAX_TASKS
VARIABLES ready, running

Init ==
  /\ ready  = TASKS
  /\ running = 0          \* 0 = idle

Start ==
  /\ running = 0
  /\ ready # {}
  /\ LET t == CHOOSE x \in ready : TRUE IN
     /\ running' = t
     /\ ready'   = ready \ {t}

Yield ==
  /\ running # 0
  /\ LET
       t  == running
       r1 == ready \cup {t}
       t2 == CHOOSE x \in r1 : TRUE
     IN
       /\ running' = t2
       /\ ready'   = r1 \ {t2}

Idle ==
  /\ ready = {}
  /\ running' = 0
  /\ UNCHANGED ready

Next == Start \/ Yield \/ Idle
Spec == 
  Init /\ [][Next]_<<ready, running>>
       /\ WF_<<ready, running>>(Yield)
       /\ WF_<<ready, running>>(Start)

\* --- Invariants (state predicates) ---
Safe == running = 0 \/ running \in TASKS

\* --- Temporal properties ---
Safety  == [](running = 0 \/ running \in TASKS)
Liveness == \A t \in TASKS : [](t \in ready => <>(running = t))
===========================
\* Modification History
\* Last modified Thu Sep 25 22:08:20 CEST 2025 by albys
\* Created Thu Sep 25 20:55:55 CEST 2025 by albys

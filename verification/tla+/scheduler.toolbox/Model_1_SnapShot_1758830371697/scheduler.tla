---- MODULE scheduler ----
EXTENDS Naturals

CONSTANT MAX_TASKS
VARIABLES tasks, ready, running

Init ==
  /\ tasks  = 1..MAX_TASKS
  /\ ready  = tasks
  /\ running = 0          \* 0 = idle

Start ==
  /\ running = 0
  /\ ready # {}
  /\ LET t == CHOOSE x \in ready : TRUE IN
     /\ running' = t
     /\ ready'   = ready \ {t}
     /\ UNCHANGED tasks

Yield ==
  /\ running # 0
  /\ LET
       t  == running
       r1 == ready \cup {t}
       t2 == CHOOSE x \in r1 : TRUE
     IN
       /\ running' = t2
       /\ ready'   = r1 \ {t2}
       /\ UNCHANGED tasks

Idle ==
  /\ ready = {}
  /\ running' = 0
  /\ UNCHANGED <<tasks, ready>>

Next == Start \/ Yield \/ Idle
Spec == Init /\ [][Next]_<<tasks, ready, running>>

Safety  == [](running = 0 \/ running \in tasks)
Liveness == \A t \in tasks : [](t \in ready => <>(running = t))
===========================
\* Modification History
\* Last modified Thu Sep 25 21:00:20 CEST 2025 by albys
\* Created Thu Sep 25 20:55:55 CEST 2025 by albys

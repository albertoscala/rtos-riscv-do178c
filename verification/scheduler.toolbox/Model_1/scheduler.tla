---- MODULE scheduler ----
EXTENDS Naturals

CONSTANT MAX_TASKS
TASKS == 1..MAX_TASKS

VARIABLES ready, running   \* running = 0 significa "idle"

\* Stato iniziale: tutti i task sono Ready, nessuno in Running
Init ==
  /\ ready   = TASKS
  /\ running = 0

\* Avvia il task t quando la CPU è idle
Start(t) ==
  /\ running = 0
  /\ t \in ready
  /\ running' = t
  /\ ready'   = ready \ {t}

\* Passa l'esecuzione al task t (il corrente torna Ready)
SwitchTo(t) ==
  /\ running # 0
  /\ t \in (ready \cup {running})
  /\ LET r1 == ready \cup {running} IN
       /\ running' = t
       /\ ready'   = r1 \ {t}

\* Nessun task pronto: CPU idle (stuttering controllato)
Idle ==
  /\ ready = {}
  /\ running' = 0
  /\ UNCHANGED ready

\* Passi possibili: avvio o switch su qualche t, oppure idle
Next ==
  \/ (\E t \in TASKS : Start(t))
  \/ (\E t \in TASKS : SwitchTo(t))
  \/ Idle

\* Specifica + fairness per evitare starvation e stuttering infinito
Spec ==
  /\ Init
  /\ [][Next]_<<ready, running>>
  /\ \A t \in TASKS : WF_<<ready, running>>(Start(t))
  /\ \A t \in TASKS : WF_<<ready, running>>(SwitchTo(t))
  /\ WF_<<ready, running>>(Next)   \* opzionale: evita stuttering infinito

\* -------- Invariants (predicati di stato) --------
Safe == running = 0 \/ running \in TASKS

\* -------- Proprietà temporali --------
Safety   == [](Safe)
Liveness == \A t \in TASKS : [](t \in ready => <>(running = t))

===========================
\* Modification History
\* Last modified Thu Sep 25 22:12:12 CEST 2025 by albys
\* Created Thu Sep 25 20:55:55 CEST 2025 by albys

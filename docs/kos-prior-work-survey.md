# Mapping KOS (Knowledge Operating System) to Prior Work: A Breadth-First Literature Survey with Emphasis on Failure Modes

## TL;DR

- **The single biggest lesson from five decades of adjacent research is that KOS’s core mechanisms map onto a *graveyard of well-intentioned systems that died for the same recurring reasons*** — design rationale capture, requirements traceability, knowledge management, model-driven engineering, the Semantic Web, and literate programming all failed primarily because the cost of capture fell on people who did not reap the benefit (asymmetric cost/benefit), formalization imposed cognitive overhead, captured knowledge decayed unmaintained, and metrics became gaming targets [@grudin1996evaluating; @shipman1999formality; @marshall2003which; @austin1996measuring].
- **KOS’s central survival bet must be that AI agents absorb the capture cost**, breaking the asymmetry (Grudin/Weber) that doomed prior systems; several concepts are well-grounded (prediction-error learning, ACT-R-style confidence decay, TMS-based tiering) but two — the **append-only graveyard of negative knowledge** and **retrodictive replay for early-warning validation** — are exactly where prior work is most damning and where KOS is methodologically most exposed to hindsight/look-ahead bias [@fischhoff1975hindsight; @weber2001intelligent].
- **Nine of the seventeen concepts warrant deeper follow-up treatment** (flagged below), chiefly because the literature identifies specific, unresolved failure modes that KOS could either fall into or, with careful design, contribute genuinely novel solutions to — especially “epistemic chaos engineering,” automated incremental formalization, and valid point-in-time retrodiction protocols.

## Key Findings

1. **The asymmetric cost/benefit problem is the number-one killer of knowledge-capture systems.** It independently sank design rationale [@grudin1996evaluating], requirements traceability [@gotel1994analysis], knowledge management, and MDE [@hutchinson2014model]. Any KOS design requiring humans to manually author graph content will replicate these deaths.
1. **Formality is costly and frequently counterproductive.** Shipman & Marshall’s “Formality Considered Harmful” [@shipman1999formality] and Marshall & Shipman’s “Which Semantic Web?” [@marshall2003which] show forced premature classification drives abandonment; *incremental formalization* is the escape hatch.
1. **Metrics on knowledge become targets** [@austin1996measuring]: any incentive attached to node counts or contribution scores will be gamed.
1. **Captured knowledge decays**, and stale-but-trusted knowledge is worse than absent knowledge (traceability decay, link rot, ACT-R base-level decay).
1. **Automation induces complacency and deskilling** [@bainbridge1983ironies; @parasuraman2010complacency] — a first-order risk for a human–AI knowledge system.
1. **Retrodictive validation is contaminated by hindsight and look-ahead bias** [@fischhoff1975hindsight] unless strictly blinded.
1. **Some knowledge is genuinely uncodifiable** [@polanyi1966tacit; @collins2010tacit], and every classification scheme creates invisible residual categories [@bowker1999sorting].

## Details

### 1. Typed knowledge graphs with epistemic confidence tiers (bedrock/frontier/graveyard)

**Supportive.** Doyle’s Truth Maintenance System [@doyle1979tms] introduced justification records so beliefs can be retracted when assumptions change; de Kleer’s ATMS [@dekleer1986atms] tracks, for each fact, the assumption sets (“environments”) under which it holds and maintains minimal inconsistent sets (“nogoods”) — a direct formal analogue of an append-only ruled-out record. AGM belief revision [@alchourron1985agm] formalizes rational belief change (expansion/contraction/revision), grounding tiered, revisable nodes.

**Critical / do-not-do.** TMS/ATMS are computationally expensive; the ATMS can blow up combinatorially in the number of environments. Do **not** attempt global logical consistency across the whole graph — de Kleer’s tractability derived from *minimal* nogoods and local reasoning. AGM’s logical omniscience and single-consistent-belief-set assumptions are unrealistic for a large, multi-author, contradiction-laden project base; treat tiers as social/epistemic labels, not a globally satisfiable logic.

**Further research. *Recommended for deeper treatment*:** formally mapping bedrock/frontier/graveyard onto belief-revision operators (a graveyard entry as a stored contraction with a justification; a “reopener” as the re-expansion condition), and combining probabilistic/possibilistic confidence with symbolic justification.

### 2. The graveyard: append-only negative knowledge

**Supportive.** Recording ruled-out approaches (approach/context/finding/ruling/reopener) is backed by the negative-results movement and organizational-learning theory, and resembles a durable decision record.

**Critical / do-not-do (priority).** This is the most dangerous concept because it is precisely what design-rationale and lessons-learned systems tried and largely failed at. Designers are documented as reluctant to record rejected alternatives (Conklin & Yakemovic). Grudin [@grudin1996evaluating] names the fatal asymmetry: capture cost falls on non-beneficiaries. At organizational scale, the GAO’s audit of NASA’s Lessons Learned Information System [@gao2002nasa] — surveying 192 managers overseeing roughly 240 programs and projects — found that “lessons are not routinely identified, collected, or shared”; managers reported that LLIS “is not the primary source for lessons learning,” instead relying on “program reviews and informal discussions with colleagues,” with the chief barriers being “lack of time to capture or submit lessons and a perception of intolerance for mistakes.” Weber, Aha & Becerra-Fernandez [@weber2001intelligent] found that despite large repositories “their information is not being used,”  and argued lessons-learned systems fail when they “introduce new processes” instead of embedding into existing workflow.

Do-not-do: (1) do not require manual, out-of-band authoring of negative entries — capture must be a by-product of existing work; (2) do not rely on cultures that punish mistakes; (3) do not assume retrieval will happen — an unqueried lesson is worthless.

**Further research. *Recommended for deeper treatment*:** whether LLM agents can auto-populate the graveyard from commit history, PR discussions, and incident postmortems (embedding capture in workflow, per Weber’s prescription), breaking the intrusiveness/asymmetry deadlock that killed prior systems.

### 3. Design rationale and argumentation structure

**Supportive.** The Toulmin model [@toulmin1958uses], Rittel’s IBIS, Conklin & Begeman’s gIBIS, and MacLean et al.‘s QOC give vocabularies for design deliberation; Dung’s abstract argumentation frameworks [@dung1995acceptability] give formal semantics for which positions are defensible under attack relations.

**Critical / do-not-do (priority).** Design rationale is the canonical cautionary tale. Buckingham Shum & Hammond [@shum1994argumentation] asked “what use at what cost?” and documented heavy capture costs for uncertain downstream benefit. Grudin [@grudin1996evaluating] documented intrusiveness and cost/benefit asymmetry. Shipman & Marshall’s “Formality Considered Harmful” [@shipman1999formality] is the key finding: forcing premature formalization causes abandonment because users resist fitting fluid ideas into rigid node/link types; their remedy is *incremental formalization* — letting structure emerge on demand.

Do-not-do: do not force up-front classification of every node/edge; do not make argument structure mandatory; do not assume rationale captured for one audience serves another.

**Further research. *Recommended for deeper treatment*:** LLM-assisted, post-hoc rationale reconstruction from communication artifacts (the “documentation perspective” Shipman & McCall favored).

### 4. Requirements traceability and spec-code correspondence

**Supportive.** Gotel & Finkelstein’s empirical analysis (100+ practitioners) [@gotel1994analysis] is foundational, distinguishing pre-RS from post-RS traceability. Reflexion Models [@murphy1995software] compare a high-level model against extracted source structure — directly relevant to spec-code correspondence. Consistency checking (xlinkit [@nentwich2003flexible]) offers scalable rule-based link checking; Perry & Wolf [@perry1992foundations] introduced architectural erosion/drift.

**Critical / do-not-do (priority).** Traceability decays and its benefit is asymmetric. Gotel & Finkelstein showed most problems stem from inadequate *pre-RS* traceability — the informal, tacit reasoning before a spec is written, exactly what is hardest to capture. The “traceability benefit problem” (Arkley & Riddle): link maintainers rarely benefit, so links go stale. DOORS-class tools are widely adopted in regulated industries but carry high maintenance burden.

Do-not-do: do not create trace links requiring manual maintenance disconnected from artifacts; do not assume link completeness; treat stale links as actively harmful (false confidence).

**Further research. *Recommended for deeper treatment*:** whether KOS’s typed spec-code links can be continuously *regenerated/validated* (IR → BERT/LLM trace-link generation is an active ICSE/RE topic) rather than manually maintained, and how to surface link decay as a first-class signal.

### 5. Documents and code as lossy projections of an underlying model

**Supportive.** Single-source-of-truth and model-driven engineering (MDE) share KOS’s premise; projectional editing and Knuth’s literate programming [@knuth1984literate] (“explaining to human beings what we want the computer to do”) articulate the vision.

**Critical / do-not-do (priority).** The empirical MDE record is a warning. Hutchinson, Whittle et al. [@hutchinson2011empirical; @hutchinson2014model] found success/failure is driven by “complex organizational, managerial and social factors,”  not technical merit; Whittle, Hutchinson & Rouncefield [@whittle2014state], surveying 450 MDE practitioners with 22 in-depth interviews, concluded that “although MDE might be more widespread than commonly believed, developers rarely use it to generate whole systems. Rather, they apply MDE to develop key parts of a system.” Round-trip engineering is notoriously brittle. Literate programming, despite technical elegance, never achieved broad adoption — tooling complexity (WEB/CWEB tied to TeX) and maintenance overhead confined it to niche reproducible-research use.

Do-not-do: do not assume the “underlying model” fully generates all projections; do not force bidirectional round-trip sync; the maintainer-vs-beneficiary asymmetry recurs here too.

**Further research. *Recommended for deeper treatment*:** treating projections as *lossy and regenerable* (compiler-like) rather than authoritative bidirectional sync.

### 6. Prediction-error learning / calibration

**Supportive.** One of KOS’s best-grounded concepts. Schultz, Dayan & Montague’s dopamine reward-prediction-error work [@schultz1997neural] and the predictive-processing / free-energy framework [@friston2010free; @clark2013whatever] establish prediction error as a fundamental learning signal, grounding “surprise_magnitude at harvest.” Forecasting-calibration research — Brier scoring, Tetlock’s *Expert Political Judgment* [@tetlock2005expert] and *Superforecasting* [@tetlock2015superforecasting] — shows calibration is measurable and trainable: in the Good Judgment Project, probability-training produced roughly a 6% Brier-score improvement in year 3 and about 7% in year 4, and Schoemaker & Tetlock separately report that as little as one hour of training improved accuracy by about 14% over a year. Pre-registration and registered reports directly support pre-registered hypotheses with predicted_confidence.

**Critical / do-not-do.** Kerr’s HARKing [@kerr1998harking] is the key hazard: if predictions are recorded *after* outcomes, calibration is meaningless. Tetlock’s own extremizing-aggregation caveat (some tournament gains may have been partly a fluke) warns against over-trusting aggregation tricks.

Do-not-do: do not allow predicted_confidence to be edited after harvest — timestamp and lock predictions; guard against Goodhart effects on calibration scores (§14).

**Further research.** Automated surprise detection triggering belief revision (links §1 and §6).

### 7. Organizational memory and knowledge management

**Supportive.** Walsh & Ungson’s organizational memory [@walsh1991organizational] and Stein & Zwass’s organizational memory information systems provide the frame; Nonaka’s SECI model [@nonaka1994dynamic] describes tacit↔explicit conversion; Wegner’s transactive memory [@wegner1987transactive] explains how groups distribute “who knows what.”

**Critical / do-not-do (priority).** The 1990s–2000s KM movement is a graveyard. Multi-case failure studies find lack of incentives and the absence of an appropriate/usable system to be the most significant barriers; KM failed when it was too IT-centric, lacked contribution incentives, and ignored culture. Nonaka’s SECI is critiqued for treating tacit→explicit conversion as unproblematic (§8).

Do-not-do: do not build a repository and assume contribution; do not ignore incentives; do not treat tacit knowledge as fully codifiable.

**Further research. *Recommended for deeper treatment*:** whether AI agents change the incentive calculus by removing the human-contribution bottleneck.

### 8. Tacit knowledge limits

**Supportive.** Polanyi’s tacit dimension [@polanyi1966tacit] (“we can know more than we can tell”) and Collins’s taxonomy [@collins2010tacit] — relational, somatic, and collective tacit knowledge  — bound what a graph can capture; Collins argues only *collective* tacit knowledge is truly uncodifiable, while relational is capturable in principle. Cook & Brown’s “knowing vs. knowledge” [@cook1999bridging] and Tsoukas’s distributed-knowledge view of the firm critique codification.

**Critical / do-not-do (priority).** Suchman’s *Plans and Situated Actions* [@suchman1987plans] shows plans are resources for action, not determinants — a warning against over-formalization. Bowker & Star’s *Sorting Things Out* [@bowker1999sorting] documents that every classification scheme produces *residual categories* (“other”) and makes some things invisible, with “torque” when lived experience does not fit — a direct warning about typed nodes and confidence tiers.

Do-not-do: do not assume the graph captures everything that matters; explicitly design for residual/uncategorizable knowledge; do not let the scheme’s blind spots become organizational blind spots.

**Further research. *Recommended for deeper treatment*:** representing the *boundary* of what is captured (metadata about what is deliberately excluded).

### 9. Human–AI collaborative cognition and distributed cognition

**Supportive.** Hutchins’s distributed cognition [@hutchins1995cognition] and Clark & Chalmers’s extended mind [@clark1998extended] frame KOS as external cognitive scaffolding; Horvitz’s mixed-initiative principles [@horvitz1999principles] guide human–AI interaction.

**Critical / do-not-do (priority).** Bainbridge’s “Ironies of Automation” [@bainbridge1983ironies] is essential: automating the routine leaves humans to handle exactly the cases automation cannot, while their skills atrophy. Parasuraman & Manzey [@parasuraman2010complacency] document automation complacency and bias — operators of consistently reliable systems are markedly worse at detecting failures, and complacency “cannot be overcome with simple practice.” 

Do-not-do: do not let KOS cause humans to stop exercising judgment about knowledge validity; guard against automation bias (trusting graph assertions because the system is usually right); preserve human domain-reasoning skill.

**Further research. *Recommended for deeper treatment*:** designing KOS to *counter* rather than induce complacency (surfacing uncertainty, forcing engagement).

### 10. Provenance and trust

**Supportive.** The W3C PROV data model [@moreau2013prov] (Entity/Activity/Agent) is the canonical provenance standard; the NATO/Admiralty source-reliability grading [@nato2016ajp21] (reliability A–F, credibility 1–6) directly parallels KOS’s source grading; Heuer’s *Psychology of Intelligence Analysis* [@heuer1999psychology] motivates structured methods to counter bias.

**Critical / do-not-do (priority).** Structured Analytic Techniques have surprisingly weak empirical validation. Dhami, Belton & Mandel [@dhami2019analysis] found analysts trained in Analysis of Competing Hypotheses “did not follow all of the steps of ACH”  and it did not reliably reduce bias. The Admiralty scale itself shows raters clustering on the A1/B2/C3 diagonal, indicating reliability and credibility are not judged independently.

Do-not-do: do not assume a structured grading scheme improves judgment; validate empirically; do not conflate source reliability with information credibility.

**Further research. *Recommended for deeper treatment*:** empirical evaluation of whether KOS’s confidence tiers actually improve decisions.

### 11. Memory architectures for LLM agents

**Supportive.** Retrieval-augmented generation [@lewis2020retrieval], Microsoft GraphRAG, MemGPT [@packer2023memgpt] (OS-inspired virtual-memory paging), and Park et al.‘s Generative Agents [@park2023generative] (memory stream + recency/importance/relevance retrieval + reflection) are directly relevant. Cognitive architectures SOAR and ACT-R [@anderson2004integrated] provide decades-tested formalizations; ACT-R’s base-level activation — activation as a function of frequency and recency  with a decay parameter (community default ~0.5)  — is an *existing formalization of “confidence half-life.”*

**Critical / do-not-do.** Ablation evidence: removing reflection in Generative Agents caused behavior to degenerate “from coherent multi-day planning to repetitive, context-free responses within 48 simulated hours”; its fixed relevance-and-recency heuristic “required extensive manual tuning” and “could not learn from memory management errors.” MemGPT’s paging can introduce latency. Memory poisoning is a documented emerging attack; catastrophic forgetting remains a continual-learning hazard.

Do-not-do: do not rely on hand-tuned retrieval heuristics; do not ignore memory-corruption/poisoning threats; do not assume more memory = better performance.

**Further research. *Recommended for deeper treatment*:** adopting ACT-R base-level activation as KOS’s principled decay/confidence model instead of an ad-hoc half-life, and defending against memory poisoning (links §15).

### 12. Knowledge decay and half-life

**Supportive.** Arbesman’s *Half-Life of Facts* [@arbesman2012halflife] documents measurable obsolescence: Poynard et al. sampled cirrhosis/hepatitis studies from 1945–1999 and had experts judge each conclusion true/false/obsolete as of 2000, yielding a reported ~45-year truth half-life (critics note this is inflated because survival was measured from publication to 2000). ACT-R base-level decay [@anderson2004integrated] gives a cognitive-science formalization. Link-rot studies find random web-page half-life of approximately two years (a 2016–17 Yahoo! Directory study likewise found ~two years), whereas a 2002 study found only ~3% of digital-library objects inaccessible after a year, implying a half-life of nearly 23 years.

**Critical / do-not-do.** Schema/ontology evolution is a known failure point — changing the schema can silently invalidate prior assertions; external references rot.

Do-not-do: do not treat any confidence assignment as permanent; version the schema; monitor external-reference decay.

**Further research.** Empirically calibrating decay rates per knowledge type (bedrock facts persist for decades; others rot in months).

### 13. Wikis / ontologies / Semantic Web as cautionary tales

**Supportive (primarily cautionary).** The Semantic Web vision and Cyc [@lenat1990cyc] represent the maximal ambition of formalized knowledge.

**Critical / do-not-do (priority).** Marshall & Shipman’s “Which Semantic Web?” [@marshall2003which] is definitive on authoring cost: “learning a knowledge representation language or tool requires the author to learn about the representation’s methods of abstraction and their effect on reasoning… Once one has learned a formal representation language, it is still often much more effort to express ideas in that representation than in a less formal representation.” The grand agent vision largely did not materialize because authoring cost and brittle ontologies were underestimated. Cyc, running since 1984, is critiqued for lacking a theoretical foundation, being unable to perform induction, and remaining “crystalline” (non-probabilistic);  despite decades and hundreds of person-years it has not yielded general common-sense AI. Enterprise wikis are frequently abandoned.

Do-not-do: do not require heavy up-front ontology engineering; do not assume contributors will learn a formal representation; do not build a brittle global ontology.

**Further research. *Recommended for deeper treatment*:** whether LLMs finally reduce the authoring-cost barrier that killed the Semantic Web.

### 14. Self-referential / reflexive systems and metrics

**Supportive.** Goodhart’s law and Campbell’s law formalize measurement dysfunction.

**Critical / do-not-do (priority).** Austin’s *Measuring and Managing Performance in Organizations* [@austin1996measuring] is the key text: when any important dimension is unmeasured, incentive-linked measurement is doomed to dysfunction as people optimize the measured at the expense of the unmeasured (Ridgway’s classic “dysfunctional consequences of performance measurements” predates it). CMM/CMMI maturity efforts spawned “process theater” — compliance activity that satisfies auditors without improving outcomes.

Do-not-do: do not attach incentives to KOS knowledge metrics (node counts, contribution scores); use metrics for informational/process improvement only, never individual performance; assume any metric will be gamed.

**Further research. *Recommended for deeper treatment*:** measuring KOS health without inducing dysfunction.

### 15. Canary / adversarial epistemics

**Supportive.** Mutation testing (DeMillo, Lipton & Sayward [@demillo1978hints]; the “coupling effect”) — deliberately injecting faults to test whether tests catch them — maps directly to injecting false knowledge to test detection. Chaos engineering (Basiri et al. [@basiri2016chaos]; Netflix’s Chaos Monkey/Kong) validates resilience by inducing failures in production. Red-teaming and data-poisoning-detection literature complete the picture.

**Critical / do-not-do.** Mutation testing is computationally expensive and suffers “equivalent mutants” (mutations that don’t change behavior, wasting analysis). Chaos engineering in production carries real risk. Injecting false knowledge into a shared graph risks contaminating it if the canary isn’t perfectly tracked and removed.

Do-not-do: do not inject adversarial knowledge without airtight provenance and rollback; beware equivalent-mutant-style false negatives.

**Further research. *Recommended for deeper treatment*:** a principled “epistemic chaos engineering” methodology for knowledge graphs — a genuinely novel contribution KOS could make.

### 16. Retrodiction and hindsight

**Supportive.** Backtesting methodology (finance) provides replay techniques; resilience engineering (Woods) and incident analysis inform replaying historical graph states.

**Critical / do-not-do (priority).** Fischhoff’s “Hindsight ≠ Foresight” [@fischhoff1975hindsight] is the essential warning: outcome knowledge inflates judged prior likelihood, and judges are “largely unaware” of the effect. Dekker [@dekker2014field] shows hindsight bias corrupts accident investigation — observers with outcome knowledge wrongly conclude operators “should have seen it.” Backtesting in finance is plagued by overfitting and look-ahead bias (accidentally using information unavailable at the replayed time).

Do-not-do: KOS’s plan to replay historical graph states to test early-warning claims is *highly vulnerable to look-ahead bias* — do not let any post-hoc information leak into the replayed state; do not evaluate early-warning claims with knowledge of what actually happened absent strict blinding.

**Further research. *Recommended for deeper treatment*:** rigorous protocols (point-in-time reconstruction, blinding) to make retrodictive early-warning tests valid — where KOS is methodologically most exposed.

### 17. Federated / distributed knowledge across repositories

**Supportive.** Conway’s law is empirically validated: MacCormack, Baldwin & Rusnak’s mirroring hypothesis [@maccormack2012exploring] found products “mirror the architectures of the organizations” that build them;  Nagappan, Murphy & Basili [@nagappan2008influence] found organizational metrics predicted failure-prone Windows Vista binaries better than code metrics.  Carlile’s framework [@carlile2004transferring] distinguishes syntactic (transfer), semantic (translation), and pragmatic (transformation) knowledge boundaries , with boundary objects [@carlile2002pragmatic] spanning them.

**Critical / do-not-do.** Carlile’s key lesson: at *pragmatic* boundaries actors have different interests, so knowledge cannot simply be transferred — it must be transformed, which is political and effortful. Federating knowledge across teams/repos will hit these boundaries; API/schema federation shows semantic mismatch across repositories.

Do-not-do: do not assume a shared schema resolves cross-team knowledge boundaries; the hardest boundaries are pragmatic (differing interests), not syntactic.

**Further research.** Boundary objects for human-AI-team knowledge federation.

## Recommendations

**Stage 1 — Design guardrails to adopt now (before building further):**

- **Make AI agents bear the capture cost.** Treat any requirement for manual human authoring of nodes/edges/graveyard entries as a red flag; capture must be a by-product of existing developer workflow (commits, PRs, incidents). *Benchmark that would change this:* if pilot data shows humans will voluntarily author ≥ the needed content without incentives, the asymmetry constraint relaxes — but the base rate from KM/DR/LLIS says they will not.
- **Enforce incremental formalization** [@shipman1999formality]: never require up-front classification; let structure emerge on demand.
- **Lock predictions.** predicted_confidence and hypotheses must be timestamped and immutable post-harvest to preserve calibration validity and prevent HARKing [@kerr1998harking].
- **Never attach incentives to knowledge metrics** [@austin1996measuring]; use them only for process feedback.

**Stage 2 — Empirical validation to prioritize:**

- Run a controlled study of whether KOS’s confidence tiers actually improve human decisions (the SAT literature warns structured schemes often do not [@dhami2019analysis]).
- Instrument graveyard *retrieval*, not just capture — an unqueried graveyard is the LLIS failure mode [@gao2002nasa; @weber2001intelligent]. *Threshold:* if graveyard entries are not queried before repeating ruled-out approaches, the feature is failing regardless of how complete it is.
- Adopt ACT-R base-level activation [@anderson2004integrated] as the principled decay model and calibrate decay rates per knowledge type against observed re-validation events.

**Stage 3 — Novel contributions to pursue (highest research upside):**

- Formalize **“epistemic chaos engineering”** (deliberate false-knowledge injection with airtight provenance/rollback) — no prior discipline exists; KOS could define it.
- Build a **valid retrodiction protocol** (point-in-time reconstruction + blinding) for early-warning claims; this is where KOS is most exposed to hindsight/look-ahead bias [@fischhoff1975hindsight] and where a rigorous method would be a real contribution.
- Test whether **LLM-borne capture and formalization** finally dissolves the cost/benefit asymmetry (Grudin/Weber gap) and the Semantic Web authoring-cost barrier (Marshall & Shipman gap).

**Research venues.** KOS naturally sits across five communities: **CSCW/CHI** (design rationale, incremental formalization, human-AI teaming, automation bias); **ICSE/FSE/RE** (traceability, MDE, architecture erosion, reflexion models); **KR/AAAI/ISWC** (TMS, belief revision, ontologies, provenance); **Organization Science/Management** (organizational memory, KM failure, boundary objects, measurement dysfunction); and **NeurIPS/ACL/agent venues** (LLM memory, GraphRAG, continual learning).

**Open questions anchored in literature gaps:** (1) Does AI-borne capture cost finally solve the asymmetric cost/benefit problem that killed design rationale and KM? (2) Can incremental formalization be automated so structure emerges without user overhead? (3) What is a valid retrodictive protocol free of hindsight/look-ahead bias? (4) Can ACT-R base-level activation serve as a principled confidence-decay model for a project knowledge graph? (5) What does epistemic chaos engineering look like as a discipline? (6) How can knowledge-graph health be measured without Goodhart dysfunction?

## Caveats

- **Search-budget limits.** A small number of concepts were sourced partly through the citation gap-filler rather than direct reading of the primary text: Conway’s-law empirical studies (§17), Carlile’s boundary framework (§17), mutation testing/chaos engineering primary papers (§15), literate-programming non-adoption (§5), and the NATO/Admiralty grading standard (§10). The bibliographic details (years, venues, DOIs) were cross-checked, but a few page ranges and edition details (e.g., AJP-2.1 exact edition/year; Dekker edition) should be verified against the publisher before publication.
- **Literate programming’s “failure” is a well-attested community narrative, not a single peer-reviewed post-mortem;** it is best framed as limited adoption outside reproducible-research niches, not a documented collapse.
- **The Admiralty A–F/1–6 six-grade form is the modern NATO standard;** the WWII original used a five-grade credibility scale. The independence critique of the scale rests on older studies (Baker et al.) and should be presented as suggestive.
- **The ~45-year clinical-fact half-life** [@arbesman2012halflife] is contested; critics note the measurement window (publication-to-2000) inflates apparent longevity. Cite it as illustrative of measurable decay, not a precise constant.
- **Some LLM-agent-memory sources are recent arXiv preprints** (MemGPT, Generative Agents ablations, memory-poisoning work) that may not be peer-reviewed in final form; the seminal items (Park et al. UIST 2023; Lewis et al. NeurIPS 2020) are archival.
- **A few relations in the task brief were reframed by the evidence:** Carlile’s 2004 paper is titled “Transferring, Translating, and Transforming” (the “Pragmatic View” title is his 2002 paper); both are cited. Where the literature was thin or contested, this is flagged in-line rather than smoothed over.

## BibTeX Bibliography

```bibtex
@article{doyle1979tms,
  author = {Doyle, Jon},
  title = {A Truth Maintenance System},
  journal = {Artificial Intelligence},
  volume = {12},
  number = {3},
  pages = {231--272},
  year = {1979},
  doi = {10.1016/0004-3702(79)90008-0}
}

@article{dekleer1986atms,
  author = {de Kleer, Johan},
  title = {An Assumption-Based {TMS}},
  journal = {Artificial Intelligence},
  volume = {28},
  number = {2},
  pages = {127--162},
  year = {1986},
  doi = {10.1016/0004-3702(86)90080-9}
}

@article{alchourron1985agm,
  author = {Alchourr{\'o}n, Carlos E. and G{\"a}rdenfors, Peter and Makinson, David},
  title = {On the Logic of Theory Change: Partial Meet Contraction and Revision Functions},
  journal = {Journal of Symbolic Logic},
  volume = {50},
  number = {2},
  pages = {510--530},
  year = {1985},
  doi = {10.2307/2274239}
}

@incollection{grudin1996evaluating,
  author = {Grudin, Jonathan},
  title = {Evaluating Opportunities for Design Capture},
  booktitle = {Design Rationale: Concepts, Techniques, and Use},
  editor = {Moran, Thomas P. and Carroll, John M.},
  publisher = {Lawrence Erlbaum Associates},
  address = {Mahwah, NJ},
  pages = {453--470},
  year = {1996}
}

@techreport{gao2002nasa,
  author = {{U.S. General Accounting Office}},
  title = {{NASA}: Better Mechanisms Needed for Sharing Lessons Learned},
  institution = {U.S. General Accounting Office},
  number = {GAO-02-195},
  address = {Washington, DC},
  year = {2002}
}

@article{weber2001intelligent,
  author = {Weber, Rosina and Aha, David W. and Becerra-Fernandez, Irma},
  title = {Intelligent Lessons Learned Systems},
  journal = {Expert Systems with Applications},
  volume = {20},
  number = {1},
  pages = {17--34},
  year = {2001},
  doi = {10.1016/S0957-4174(00)00046-4}
}

@book{toulmin1958uses,
  author = {Toulmin, Stephen},
  title = {The Uses of Argument},
  publisher = {Cambridge University Press},
  address = {Cambridge, UK},
  year = {1958}
}

@article{dung1995acceptability,
  author = {Dung, Phan Minh},
  title = {On the Acceptability of Arguments and Its Fundamental Role in Nonmonotonic Reasoning, Logic Programming and n-Person Games},
  journal = {Artificial Intelligence},
  volume = {77},
  number = {2},
  pages = {321--357},
  year = {1995},
  doi = {10.1016/0004-3702(94)00041-X}
}

@article{shum1994argumentation,
  author = {Buckingham Shum, Simon and Hammond, Nick},
  title = {Argumentation-Based Design Rationale: What Use at What Cost?},
  journal = {International Journal of Human-Computer Studies},
  volume = {40},
  number = {4},
  pages = {603--652},
  year = {1994},
  doi = {10.1006/ijhc.1994.1029}
}

@article{shipman1999formality,
  author = {Shipman, Frank M. and Marshall, Catherine C.},
  title = {Formality Considered Harmful: Experiences, Emerging Themes, and Directions on the Use of Formal Representations in Interactive Systems},
  journal = {Computer Supported Cooperative Work},
  volume = {8},
  number = {4},
  pages = {333--352},
  year = {1999},
  doi = {10.1023/A:1008716330212}
}

@inproceedings{gotel1994analysis,
  author = {Gotel, Orlena C. Z. and Finkelstein, Anthony C. W.},
  title = {An Analysis of the Requirements Traceability Problem},
  booktitle = {Proceedings of the First International Conference on Requirements Engineering (ICRE '94)},
  pages = {94--101},
  year = {1994},
  doi = {10.1109/ICRE.1994.292398}
}

@article{murphy1995software,
  author = {Murphy, Gail C. and Notkin, David and Sullivan, Kevin},
  title = {Software Reflexion Models: Bridging the Gap Between Source and High-Level Models},
  journal = {ACM SIGSOFT Software Engineering Notes},
  volume = {20},
  number = {4},
  pages = {18--28},
  year = {1995},
  doi = {10.1145/222132.222136}
}

@article{nentwich2003flexible,
  author = {Nentwich, Christian and Capra, Licia and Emmerich, Wolfgang and Finkelstein, Anthony},
  title = {xlinkit: A Consistency Checking and Smart Link Generation Service},
  journal = {ACM Transactions on Internet Technology},
  volume = {2},
  number = {2},
  pages = {151--185},
  year = {2002},
  doi = {10.1145/514183.514186}
}

@article{perry1992foundations,
  author = {Perry, Dewayne E. and Wolf, Alexander L.},
  title = {Foundations for the Study of Software Architecture},
  journal = {ACM SIGSOFT Software Engineering Notes},
  volume = {17},
  number = {4},
  pages = {40--52},
  year = {1992},
  doi = {10.1145/141874.141884}
}

@article{knuth1984literate,
  author = {Knuth, Donald E.},
  title = {Literate Programming},
  journal = {The Computer Journal},
  volume = {27},
  number = {2},
  pages = {97--111},
  year = {1984},
  doi = {10.1093/comjnl/27.2.97}
}

@inproceedings{hutchinson2011empirical,
  author = {Hutchinson, John and Whittle, Jon and Rouncefield, Mark and Kristoffersen, Steinar},
  title = {Empirical Assessment of {MDE} in Industry},
  booktitle = {Proceedings of the 33rd International Conference on Software Engineering (ICSE '11)},
  pages = {471--480},
  year = {2011},
  doi = {10.1145/1985793.1985858}
}

@article{hutchinson2014model,
  author = {Hutchinson, John and Whittle, Jon and Rouncefield, Mark},
  title = {Model-Driven Engineering Practices in Industry: Social, Organizational and Managerial Factors That Lead to Success or Failure},
  journal = {Science of Computer Programming},
  volume = {89},
  pages = {144--161},
  year = {2014},
  doi = {10.1016/j.scico.2013.03.017}
}

@article{whittle2014state,
  author = {Whittle, Jon and Hutchinson, John and Rouncefield, Mark},
  title = {The State of Practice in Model-Driven Engineering},
  journal = {IEEE Software},
  volume = {31},
  number = {3},
  pages = {79--85},
  year = {2014},
  doi = {10.1109/MS.2013.65}
}

@article{schultz1997neural,
  author = {Schultz, Wolfram and Dayan, Peter and Montague, P. Read},
  title = {A Neural Substrate of Prediction and Reward},
  journal = {Science},
  volume = {275},
  number = {5306},
  pages = {1593--1599},
  year = {1997},
  doi = {10.1126/science.275.5306.1593}
}

@article{friston2010free,
  author = {Friston, Karl},
  title = {The Free-Energy Principle: A Unified Brain Theory?},
  journal = {Nature Reviews Neuroscience},
  volume = {11},
  number = {2},
  pages = {127--138},
  year = {2010},
  doi = {10.1038/nrn2787}
}

@article{clark2013whatever,
  author = {Clark, Andy},
  title = {Whatever Next? Predictive Brains, Situated Agents, and the Future of Cognitive Science},
  journal = {Behavioral and Brain Sciences},
  volume = {36},
  number = {3},
  pages = {181--204},
  year = {2013},
  doi = {10.1017/S0140525X12000477}
}

@book{tetlock2005expert,
  author = {Tetlock, Philip E.},
  title = {Expert Political Judgment: How Good Is It? How Can We Know?},
  publisher = {Princeton University Press},
  address = {Princeton, NJ},
  year = {2005}
}

@book{tetlock2015superforecasting,
  author = {Tetlock, Philip E. and Gardner, Dan},
  title = {Superforecasting: The Art and Science of Prediction},
  publisher = {Crown},
  address = {New York},
  year = {2015}
}

@article{kerr1998harking,
  author = {Kerr, Norbert L.},
  title = {HARKing: Hypothesizing After the Results Are Known},
  journal = {Personality and Social Psychology Review},
  volume = {2},
  number = {3},
  pages = {196--217},
  year = {1998},
  doi = {10.1207/s15327957pspr0203_4}
}

@article{walsh1991organizational,
  author = {Walsh, James P. and Ungson, Gerardo Rivera},
  title = {Organizational Memory},
  journal = {Academy of Management Review},
  volume = {16},
  number = {1},
  pages = {57--91},
  year = {1991},
  doi = {10.2307/258607}
}

@article{nonaka1994dynamic,
  author = {Nonaka, Ikujiro},
  title = {A Dynamic Theory of Organizational Knowledge Creation},
  journal = {Organization Science},
  volume = {5},
  number = {1},
  pages = {14--37},
  year = {1994},
  doi = {10.1287/orsc.5.1.14}
}

@incollection{wegner1987transactive,
  author = {Wegner, Daniel M.},
  title = {Transactive Memory: A Contemporary Analysis of the Group Mind},
  booktitle = {Theories of Group Behavior},
  editor = {Mullen, Brian and Goethals, George R.},
  publisher = {Springer},
  address = {New York},
  pages = {185--208},
  year = {1987},
  doi = {10.1007/978-1-4612-4634-3_9}
}

@book{polanyi1966tacit,
  author = {Polanyi, Michael},
  title = {The Tacit Dimension},
  publisher = {Doubleday},
  address = {Garden City, NY},
  year = {1966}
}

@book{collins2010tacit,
  author = {Collins, Harry},
  title = {Tacit and Explicit Knowledge},
  publisher = {University of Chicago Press},
  address = {Chicago},
  year = {2010}
}

@article{cook1999bridging,
  author = {Cook, Scott D. N. and Brown, John Seely},
  title = {Bridging Epistemologies: The Generative Dance Between Organizational Knowledge and Organizational Knowing},
  journal = {Organization Science},
  volume = {10},
  number = {4},
  pages = {381--400},
  year = {1999},
  doi = {10.1287/orsc.10.4.381}
}

@book{suchman1987plans,
  author = {Suchman, Lucy A.},
  title = {Plans and Situated Actions: The Problem of Human-Machine Communication},
  publisher = {Cambridge University Press},
  address = {Cambridge, UK},
  year = {1987}
}

@book{bowker1999sorting,
  author = {Bowker, Geoffrey C. and Star, Susan Leigh},
  title = {Sorting Things Out: Classification and Its Consequences},
  publisher = {MIT Press},
  address = {Cambridge, MA},
  year = {1999}
}

@book{hutchins1995cognition,
  author = {Hutchins, Edwin},
  title = {Cognition in the Wild},
  publisher = {MIT Press},
  address = {Cambridge, MA},
  year = {1995}
}

@article{clark1998extended,
  author = {Clark, Andy and Chalmers, David},
  title = {The Extended Mind},
  journal = {Analysis},
  volume = {58},
  number = {1},
  pages = {7--19},
  year = {1998},
  doi = {10.1093/analys/58.1.7}
}

@inproceedings{horvitz1999principles,
  author = {Horvitz, Eric},
  title = {Principles of Mixed-Initiative User Interfaces},
  booktitle = {Proceedings of the SIGCHI Conference on Human Factors in Computing Systems (CHI '99)},
  pages = {159--166},
  year = {1999},
  doi = {10.1145/302979.303030}
}

@article{bainbridge1983ironies,
  author = {Bainbridge, Lisanne},
  title = {Ironies of Automation},
  journal = {Automatica},
  volume = {19},
  number = {6},
  pages = {775--779},
  year = {1983},
  doi = {10.1016/0005-1098(83)90046-8}
}

@article{parasuraman2010complacency,
  author = {Parasuraman, Raja and Manzey, Dietrich H.},
  title = {Complacency and Bias in Human Use of Automation: An Attentional Integration},
  journal = {Human Factors},
  volume = {52},
  number = {3},
  pages = {381--410},
  year = {2010},
  doi = {10.1177/0018720810376055}
}

@techreport{moreau2013prov,
  author = {Moreau, Luc and Missier, Paul},
  title = {{PROV-DM}: The {PROV} Data Model},
  institution = {World Wide Web Consortium (W3C)},
  type = {W3C Recommendation},
  year = {2013},
  note = {30 April 2013}
}

@book{nato2016ajp21,
  author = {{NATO Standardization Office}},
  title = {{AJP-2.1}: Allied Joint Doctrine for Intelligence Procedures, Edition A, Version 1},
  publisher = {NATO Standardization Office},
  year = {2016}
}

@book{heuer1999psychology,
  author = {Heuer, Richards J.},
  title = {Psychology of Intelligence Analysis},
  publisher = {Center for the Study of Intelligence, CIA},
  address = {Washington, DC},
  year = {1999}
}

@article{dhami2019analysis,
  author = {Dhami, Mandeep K. and Belton, Ian K. and Mandel, David R.},
  title = {The "Analysis of Competing Hypotheses" in Intelligence Analysis},
  journal = {Applied Cognitive Psychology},
  volume = {33},
  number = {6},
  pages = {1080--1090},
  year = {2019},
  doi = {10.1002/acp.3550}
}

@inproceedings{lewis2020retrieval,
  author = {Lewis, Patrick and Perez, Ethan and Piktus, Aleksandra and Petroni, Fabio and Karpukhin, Vladimir and Goyal, Naman and K{\"u}ttler, Heinrich and Lewis, Mike and Yih, Wen-tau and Rockt{\"a}schel, Tim and Riedel, Sebastian and Kiela, Douwe},
  title = {Retrieval-Augmented Generation for Knowledge-Intensive {NLP} Tasks},
  booktitle = {Advances in Neural Information Processing Systems 33 (NeurIPS 2020)},
  year = {2020}
}

@article{packer2023memgpt,
  author = {Packer, Charles and Wooders, Sarah and Lin, Kevin and Fang, Vivian and Patil, Shishir G. and Stoica, Ion and Gonzalez, Joseph E.},
  title = {{MemGPT}: Towards {LLMs} as Operating Systems},
  journal = {arXiv preprint arXiv:2310.08560},
  year = {2023}
}

@inproceedings{park2023generative,
  author = {Park, Joon Sung and O'Brien, Joseph C. and Cai, Carrie J. and Morris, Meredith Ringel and Liang, Percy and Bernstein, Michael S.},
  title = {Generative Agents: Interactive Simulacra of Human Behavior},
  booktitle = {Proceedings of the 36th Annual ACM Symposium on User Interface Software and Technology (UIST '23)},
  pages = {1--22},
  year = {2023},
  doi = {10.1145/3586183.3606763}
}

@article{anderson2004integrated,
  author = {Anderson, John R. and Bothell, Daniel and Byrne, Michael D. and Douglass, Scott and Lebiere, Christian and Qin, Yulin},
  title = {An Integrated Theory of the Mind},
  journal = {Psychological Review},
  volume = {111},
  number = {4},
  pages = {1036--1060},
  year = {2004},
  doi = {10.1037/0033-295X.111.4.1036}
}

@book{arbesman2012halflife,
  author = {Arbesman, Samuel},
  title = {The Half-Life of Facts: Why Everything We Know Has an Expiration Date},
  publisher = {Current/Penguin},
  address = {New York},
  year = {2012}
}

@article{lenat1990cyc,
  author = {Lenat, Douglas B. and Guha, R. V. and Pittman, Karen and Pratt, Dexter and Shepherd, Mary},
  title = {Cyc: Toward Programs with Common Sense},
  journal = {Communications of the ACM},
  volume = {33},
  number = {8},
  pages = {30--49},
  year = {1990},
  doi = {10.1145/79173.79176}
}

@inproceedings{marshall2003which,
  author = {Marshall, Catherine C. and Shipman, Frank M.},
  title = {Which Semantic Web?},
  booktitle = {Proceedings of the Fourteenth ACM Conference on Hypertext and Hypermedia (HT '03)},
  pages = {57--66},
  year = {2003},
  doi = {10.1145/900051.900063}
}

@book{austin1996measuring,
  author = {Austin, Robert D.},
  title = {Measuring and Managing Performance in Organizations},
  publisher = {Dorset House},
  address = {New York},
  year = {1996}
}

@article{demillo1978hints,
  author = {DeMillo, Richard A. and Lipton, Richard J. and Sayward, Frederick G.},
  title = {Hints on Test Data Selection: Help for the Practicing Programmer},
  journal = {Computer},
  volume = {11},
  number = {4},
  pages = {34--41},
  year = {1978},
  doi = {10.1109/C-M.1978.218136}
}

@article{basiri2016chaos,
  author = {Basiri, Ali and Behnam, Niosha and de Rooij, Ruud and Hochstein, Lorin and Kosewski, Luke and Reynolds, Justin and Rosenthal, Casey},
  title = {Chaos Engineering},
  journal = {IEEE Software},
  volume = {33},
  number = {3},
  pages = {35--41},
  year = {2016},
  doi = {10.1109/MS.2016.60}
}

@article{fischhoff1975hindsight,
  author = {Fischhoff, Baruch},
  title = {Hindsight $\neq$ Foresight: The Effect of Outcome Knowledge on Judgment Under Uncertainty},
  journal = {Journal of Experimental Psychology: Human Perception and Performance},
  volume = {1},
  number = {3},
  pages = {288--299},
  year = {1975},
  doi = {10.1037/0096-1523.1.3.288}
}

@book{dekker2014field,
  author = {Dekker, Sidney},
  title = {The Field Guide to Understanding 'Human Error'},
  edition = {3rd},
  publisher = {Ashgate},
  address = {Farnham, UK},
  year = {2014}
}

@article{maccormack2012exploring,
  author = {MacCormack, Alan and Baldwin, Carliss and Rusnak, John},
  title = {Exploring the Duality Between Product and Organizational Architectures: A Test of the "Mirroring" Hypothesis},
  journal = {Research Policy},
  volume = {41},
  number = {8},
  pages = {1309--1324},
  year = {2012},
  doi = {10.1016/j.respol.2012.04.011}
}

@inproceedings{nagappan2008influence,
  author = {Nagappan, Nachiappan and Murphy, Brendan and Basili, Victor R.},
  title = {The Influence of Organizational Structure on Software Quality: An Empirical Case Study},
  booktitle = {Proceedings of the 30th International Conference on Software Engineering (ICSE '08)},
  pages = {521--530},
  year = {2008},
  doi = {10.1145/1368088.1368160}
}

@article{carlile2004transferring,
  author = {Carlile, Paul R.},
  title = {Transferring, Translating, and Transforming: An Integrative Framework for Managing Knowledge Across Boundaries},
  journal = {Organization Science},
  volume = {15},
  number = {5},
  pages = {555--568},
  year = {2004},
  doi = {10.1287/orsc.1040.0094}
}

@article{carlile2002pragmatic,
  author = {Carlile, Paul R.},
  title = {A Pragmatic View of Knowledge and Boundaries: Boundary Objects in New Product Development},
  journal = {Organization Science},
  volume = {13},
  number = {4},
  pages = {442--455},
  year = {2002},
  doi = {10.1287/orsc.13.4.442.2953}
}
```
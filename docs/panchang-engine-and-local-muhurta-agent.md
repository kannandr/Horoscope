# Panchang Engine And Local Muhurta Agent Roadmap

## Executive Summary

The current Rust engine covers the five basic Panchang angas:

- tithi
- vaara
- nakshatra
- yoga
- karana

It also includes sidereal Sun/Moon longitude, sunrise/sunset, hora, rashi,
month cells, civil-day tithi/nakshatra/yoga/karana segments, a richer
Panchang-day response, Rahu Kalam, Yama Gandam, Gulika Kalam, Abhijit
Muhurta, and basic Tamil solar calendar metadata.

For a sophisticated Indian/Tamil Panchang and muhurta platform, the next step is
not to "teach" an LLM to calculate Panchang. `panchang-core` must remain the
authority for astronomy and calendar boundaries. `muhurta-engine` should remain
the authority for rule evaluation. A local model should parse natural language,
ask missing questions, call the Panchang MCP and muhurta API surfaces, and
explain deterministic Rust output in a natural voice.

## Current Coverage

### Covered In Rust

- Local time to UTC and Julian day conversion.
- Timezone-aware requests.
- Sidereal Sun and Moon positions.
- Ayanamsha selection: Lahiri, Raman, and a placeholder Lahiri-alt.
- Tithi index, paksha, paksha day, and tithi name.
- Nakshatra index, Tamil nakshatra name, and pada.
- Yoga index/name.
- Karana name plus current karana start/end in snapshot output.
- Sun and Moon rashi.
- Vaara by local civil weekday.
- Sunrise, sunset, next sunrise.
- Hora table.
- Tithi/nakshatra/yoga/karana civil-day intervals.
- Rich Panchang-day response with civil-midnight and sunrise-day modes.
- Rahu Kalam, Yama Gandam, Gulika Kalam, and Abhijit Muhurta.
- Basic Tamil calendar metadata: solar month, year name, ayana, ritu, weekday.
- Month view day cells.
- MCP tools for snapshot, civil-day segments, Panchang day, and inauspicious periods.
- Separate `muhurta-engine` / `muhurta-api` for auspicious-time scoring.

### Important Gaps

- No durmuhurta, varjyam, amrita kalam, or simple nalla neram/gowri panchangam blocks.
- No moonrise/moonset.
- Tamil solar calendar layer is started, but not complete: sankranti,
  paksha labels in Tamil, festival-ready day metadata, and Tamil-script labels
  are still missing.
- No personal astrology layer: janma nakshatra, janma rashi, tara bala,
  chandra bala, chandrashtama, panchaka, ganda moola, vedha, or event-specific
  compatibility rules.
- No horoscope/jataka layer: graha positions beyond Sun/Moon, lagna, houses,
  varga charts, dasha, retrograde status, or transit context.
- Muhurta search is currently a simple hourly scorer in `muhurta-engine`. It is
  not yet a real rule-pack engine.
- The MCP layer intentionally exposes calculation tools only. It does not
  expose rule catalogs, personal-context evaluation, or a natural-language
  muhurta workflow.

## Engine Hardening Plan

### Phase 1: Panchang Day Completeness

Add a richer day model:

- `PanchangDayRequest`
  - date
  - timezone
  - latitude/longitude
  - ayanamsha
  - engine
  - day mode: `civil_midnight` or `sunrise_day`
- `PanchangDayResponse`
  - sunrise/sunset/next sunrise
  - vaara civil and vaara at sunrise
  - tithi intervals
  - nakshatra intervals
  - yoga intervals
  - karana intervals
  - hora intervals
  - rahu/yama/gulika
  - abhijit/durmuhurta
  - moonrise/moonset when available

Keep the current `civil-day` API for UI compatibility, but add a specialist
`panchang-day` endpoint for Panchang and muhurta work.

### Phase 2: Tamil Panchang Layer

Add Tamil calendar output:

- Tamil month from sidereal solar ingress.
- Tamil year name.
- Ayana and ritu.
- Tamil names for tithi, yoga, karana, rashi, weekday.
- Tamil nakshatra is already present but should be normalized and documented.
- Sunrise-day labels: tithi/nakshatra/yoga/karana at sunrise.
- Common Tamil almanac blocks:
  - rahu kalam
  - yama gandam
  - kuligai/gulika
  - nalla neram
  - gowri panchangam
  - chandrashtama by moon sign relative to janma rashi

### Phase 3: High-Confidence Astronomy

Keep all calculations local in Rust.

Recommended structure:

- `EphemerisProvider` trait.
- Built-in deterministic providers:
  - current Meeus provider for lightweight Panchang.
  - high-precision local provider for public beta.
- Golden tests against locked reference data for Chennai, Bengaluru, Livermore,
  DST days, amavasya/purnima boundaries, and nakshatra wrap days.

For horoscope-grade work, add graha positions:

- Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn.
- Rahu/Ketu mean and true node options.
- Retrograde status.
- Lagna and houses.
- Divisional charts starting with Navamsa.

### Phase 4: Rule Engine Instead Of Hardcoded Scoring

Move muhurta rules out of ad hoc Rust conditionals and into versioned rule
packs.

Example rule pack structure:

```text
rules/
  tamil_general.toml
  marriage.toml
  gruhapravesam.toml
  naming.toml
  travel.toml
  business_opening.toml
  education.toml
```

Rules should support:

- hard exclusions
- soft exclusions
- positive scoring
- required windows
- event-specific presets
- person-specific checks
- explanation text
- source/citation notes for future review

The Rust engine should output:

- candidate windows
- score
- confidence
- hard exclusions
- soft cautions
- positive reasons
- missing user data
- exact local timestamps
- source rule IDs

These rule-pack outputs belong in `muhurta-engine`, not `panchang-core`.
`panchang-core` should continue to provide facts only.

## Personal Muhurta Inputs

The natural-language workflow should extract a structured request:

```json
{
  "event_type": "house_warming",
  "date_start": "2026-06-01",
  "date_end": "2026-06-30",
  "location": {
    "address": "Livermore, CA",
    "timezone": "America/Los_Angeles",
    "latitude": 37.6821,
    "longitude": -121.768
  },
  "participants": [
    {
      "role": "primary",
      "janma_nakshatra": "Swati",
      "janma_rashi": "Tula"
    }
  ],
  "constraints": {
    "minimum_duration_minutes": 60,
    "daytime_only": true,
    "preferred_weekdays": ["Thursday", "Friday"]
  }
}
```

If the user does not know their janma nakshatra or rashi, the model should ask
for either:

- birth date/time/place, so the horoscope engine can calculate it locally; or
- a known nakshatra/rashi provided by the user.

## MCP Roadmap

Add these MCP tools:

Current calculation tools:

- `calculate_panchang_snapshot`
- `list_civil_day_segments`
- `calculate_panchang_day`
- `list_inauspicious_periods`

Candidate future calculation tools:

- `list_panchang_intervals`
- `calculate_tamil_calendar_day`
- `calculate_birth_profile`

Do not put final muhurta scoring back into `panchang-mcp`. Keep
`evaluate_muhurta_rules`, `search_personalized_muhurta`, and
`explain_muhurta_rules` in `muhurta-api` or a future local agent layer that
calls `panchang-mcp`.

## Local Model Recommendation

Recommended default: **Qwen3-8B** locally via Ollama or llama.cpp.

Why:

- Apache 2.0 license.
- Strong reasoning mode and non-thinking mode.
- Strong multilingual support, useful for English/Tamil/Sanskrit naming.
- Documented tool/agent capability.
- Runs locally on practical hardware when quantized.

Smaller edge option:

- **Qwen3-4B** for laptops or smaller edge boxes.

Higher-quality local option:

- **Qwen3-14B** or **Qwen3-30B-A3B** when memory/GPU capacity allows.

Runtime options:

- Ollama for the simplest local deployment and tool-calling loop.
- llama.cpp / llama-server for a lean, OpenAI-compatible local inference server.
- Qwen-Agent if we want built-in MCP configuration support around Qwen models.

Sources:

- Qwen3 model cards: https://huggingface.co/Qwen/Qwen3-8B and https://huggingface.co/Qwen/Qwen3-4B
- Ollama tool calling: https://docs.ollama.com/capabilities/tool-calling
- Qwen-Agent MCP guide: https://qwenlm.github.io/Qwen-Agent/en/guide/core_moduls/mcp/
- llama.cpp function calling: https://github.com/ggml-org/llama.cpp/blob/master/docs/function-calling.md

## Local Agent Architecture

Add a new local service:

```text
panchang-agent
  input: natural language request
  model: local Qwen3 via Ollama or llama.cpp
  tools: Panchang MCP
  output: structured request, candidate windows, explanation
```

Flow:

1. User asks in natural language.
2. Agent extracts event type, location, date range, participant details, and
   constraints.
3. Agent asks follow-up questions for missing critical fields.
4. Agent calls MCP tools.
5. Rust rule engine scores windows.
6. Agent explains results without inventing calculations.

The LLM is not allowed to compute tithi, nakshatra, rahu kalam, or muhurta
rules from memory. It must call tools or quote rule-engine output.

## Training Strategy

Do not fine-tune first.

Start with:

- deterministic Rust rule packs;
- strict JSON schemas;
- tool-calling prompts;
- examples for event types and missing-field questions.

Fine-tune later only if needed:

- LoRA/SFT on synthetic examples generated from the rule packs.
- Task 1: natural language to `PersonalMuhurtaRequest` JSON.
- Task 2: rule output to clear customer explanation.
- Task 3: ask a concise follow-up question when required fields are missing.

Evaluation must be deterministic:

- exact JSON schema validity;
- correct tool selection;
- no fabricated auspicious windows;
- no hidden rule claims outside the rule pack;
- timezone correctness;
- explanation matches returned rule IDs.

## Immediate Implementation Order

1. Add `PanchangDayResponse` with yoga/karana intervals and bad-period blocks. Done in initial Rust slice.
2. Add rahu kalam, yama gandam, and gulika calculations. Done in initial Rust slice.
3. Expose the new day and inauspicious-period tools through MCP. Done in initial Rust slice.
4. Add Tamil calendar names and Tamil day metadata. Started in initial Rust slice with solar month, year name, ayana, ritu, and Tamil weekday.
5. Replace the current muhurta scorer with rule-pack evaluation. Started by feeding rahu/yama/gulika and abhijit blocks into the current scorer; full rule packs remain next.
6. Add personalized muhurta input types.
7. Add a local agent prototype using Qwen3-8B with Ollama.
8. Build golden fixtures and specialist test cases before public beta.

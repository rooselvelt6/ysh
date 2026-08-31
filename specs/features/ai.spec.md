# AI Engine — Motor de Inteligencia Artificial

## Overview

Motor de IA para moderación de contenido, detección de anomalías, matching de usuarios, redes neuronales, y optimización genética. Implementado 100% en Rust (sin dependencias de Python/ONNX).

**Base path:** `/api/v1`
**Auth:** Requerida

---

## Endpoints

### POST /ai/moderation/text — Moderar texto

- **Auth:** Requerida
- **Request:**
```json
{
  "content": "string",
  "author_id?": "i64"
}
```
- **Response 200:**
```json
{
  "decision": "string (allow|flag|block)",
  "severity": "f64 (0.0-1.0)",
  "categories": ["string[]"],
  "matches": ["string[]"]
}
```
- **Reglas:** Block → envía a AI actor para procesamiento asíncrono

---

### POST /ai/anomaly/score — Score de anomalía

- **Auth:** Requerida
- **Request:**
```json
{
  "features": ["f64[]"]
}
```
- **Response 200:**
```json
{
  "score": "f64",
  "flagged": "bool"
}
```
- **Reglas:** flagged si score >= threshold (config `anomaly_flag_threshold`)

---

### POST /ai/anomaly/detector — Detector streaming

- **Auth:** Requerida
- **Request:**
```json
{
  "series": [["f64[]"]]
}
```
- **Response 200:**
```json
{
  "anomalous_indices": ["i64[]"],
  "count": "i64"
}
```
- **Reglas:** Streaming Z-score detector

---

### POST /ai/matching/score — Score de compatibilidad

- **Auth:** Requerida
- **Request:**
```json
{
  "a": ["f64[]"],
  "b": ["f64[]"]
}
```
- **Response 200:** `{ "score": "f64" }`
- **Reglas:** vectores deben tener igual longitud

---

### POST /ai/matching/vectorize — Vectorizar perfil

- **Auth:** Requerida
- **Request:**
```json
{
  "interests": "string[]",
  "region": "string",
  "age": "i32",
  "gender": "string",
  "rating": "f64"
}
```
- **Response 200:**
```json
{
  "vector": ["f64[]"]
}
```

---

### POST /ai/neural/predict — Predicción neuronal

- **Auth:** Requerida
- **Request:** `{ "input": ["f64[]"] }`
- **Response 200:** `{ "prediction": "f64" }`
- **Nota:** pesos aleatorios cada llamada (demo)

---

### POST /ai/neural/train — Entrenar red neuronal

- **Auth:** Requerida
- **Request:**
```json
{
  "samples": [
    {
      "input": ["f64[]"],
      "target": "f64"
    }
  ]
}
```
- **Response 200:**
```json
{
  "loss": "f64",
  "trained": true
}
```
- **Reglas:** 500 épocas, inputs deben tener misma longitud

---

### POST /ai/optimize/genetic — Optimización genética

- **Auth:** Requerida
- **Request:** `{ "dims?": "i32 (default: 4)" }`
- **Response 200:**
```json
{
  "best_genome": ["f64[]"],
  "best_fitness": "f64"
}
```

---

### GET /ai/stats — Estadísticas del motor AI

- **Auth:** No requerida
- **Response 200:**
```json
{
  "enabled": "bool",
  "models": ["string[]"],
  "stats": "object"
}
```

---

## Configuración

```toml
[ai]
enabled = true
text_moderation_sensitivity = 0.6
text_moderation_flag_threshold = 0.45
anomaly_flag_threshold = 0.7
anomaly_std_devs = 3.0
matching_score_scale = 1.0
neural_input_size = 4
neural_hidden_size = 8
genetic_population_size = 50
genetic_generations = 30
genetic_mutation_rate = 0.1
annealing_start_temp = 10.0
annealing_cooling_factor = 0.995
annealing_iterations = 1000
annealing_step_size = 0.2
auto_report_on_block = true
```

---

## Dependencies

- **Moments:** auto-moderación al crear posts
- **Chat:** auto-moderación de mensajes
- **Moderation:** decisiones AI alimentan la cola de moderación

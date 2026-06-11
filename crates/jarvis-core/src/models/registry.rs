use std::any::Any;
  use std::collections::HashMap;
  use std::sync::Arc;
  use parking_lot::{Mutex, RwLock};

  use super::structs::ModelDef;

  /// Central model registry. Loads models once and shares them between components.
  /// Thread-safe: concurrent requests for the same model load it exactly once.
  pub struct ModelRegistry {
      loaded: Mutex<HashMap<String, Arc<dyn Any + Send + Sync>>>,
      catalog: RwLock<Vec<ModelDef>>,
  }

  impl ModelRegistry {
      pub fn new() -> Self {
          Self {
              loaded: Mutex::new(HashMap::new()),
              catalog: RwLock::new(Vec::new()),
          }
      }

      pub fn set_catalog(&self, defs: Vec<ModelDef>) {
          *self.catalog.write() = defs;
      }

      pub fn with_catalog<R>(&self, f: impl FnOnce(&[ModelDef]) -> R) -> R {
          f(&self.catalog.read())
      }

      pub fn get_model_def(&self, id: &str) -> Option<ModelDef> {
          self.catalog.read().iter().find(|m| m.id == id).cloned()
      }

      /// Get a loaded model, downcasted to the expected type.
      pub fn get<T: 'static + Send + Sync>(&self, id: &str) -> Option<Arc<T>> {
          self.loaded.lock()
              .get(id)?
              .clone()
              .downcast::<T>()
              .ok()
      }

      /// Insert an already-constructed model into the registry.
      pub fn insert<T: 'static + Send + Sync>(&self, id: &str, model: T) -> Arc<T> {
          let arc = Arc::new(model);
          self.loaded.lock().insert(id.to_string(), arc.clone());
          arc
      }

      /// Get or load a model. Guaranteed to load exactly once even under concurrent access.
      /// The loader is called while holding the loaded-map lock, which prevents the TOCTOU
      /// race where two threads both see "not loaded" and both start loading.
      /// Loaders are expected to be fast relative to model-load time; if a loader needs to
      /// call back into the registry for a dependency, use a separate registry instance.
      pub fn get_or_load<T: 'static + Send + Sync>(
          &self,
          id: &str,
          loader: impl FnOnce(&ModelDef) -> Result<T, String>,
      ) -> Result<Arc<T>, String> {
          // Hold the map lock for the entire load to prevent double-load.
          // This is safe because loaders do not re-enter the same registry.
          let mut map = self.loaded.lock();

          // Fast path: already loaded
          if let Some(existing) = map.get(id) {
              if let Ok(arc) = existing.clone().downcast::<T>() {
                  info!("Model '{}' already loaded, reusing", id);
                  return Ok(arc);
              }
          }

          // Slow path: load the model
          let def = self.catalog.read().iter().find(|m| m.id == id).cloned()
              .ok_or_else(|| format!("Model '{}' not found in catalog", id))?;

          info!("Loading model '{}'...", id);
          let model = loader(&def)?;
          let arc = Arc::new(model) as Arc<dyn Any + Send + Sync>;
          map.insert(id.to_string(), arc.clone());

          arc.downcast::<T>()
              .map_err(|_| format!("Type mismatch after loading model '{}'", id))
      }
  }
  
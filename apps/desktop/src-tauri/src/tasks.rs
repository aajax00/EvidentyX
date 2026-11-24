//! Tâches longues (empreinte, analyse) : progression et annulation.
//!
//! L'interface choisit un identifiant de tâche, reçoit la progression par un
//! canal Tauri et peut demander l'annulation avec `cancel_task`.

use serde::Serialize;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::ipc::Channel;

/// Indicateurs d'annulation des tâches en cours.
#[derive(Default)]
pub struct Tasks(Mutex<HashMap<String, Arc<AtomicBool>>>);

impl Tasks {
    /// Enregistre une tâche ; elle est retirée quand le jeton est libéré.
    pub fn start(&self, id: String) -> TaskToken<'_> {
        let flag = Arc::new(AtomicBool::new(false));
        self.lock().insert(id.clone(), Arc::clone(&flag));
        TaskToken {
            tasks: self,
            id,
            flag,
        }
    }

    /// `false` si la tâche est inconnue (déjà terminée).
    pub fn cancel(&self, id: &str) -> bool {
        self.lock()
            .get(id)
            .map(|f| f.store(true, Ordering::Relaxed))
            .is_some()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<String, Arc<AtomicBool>>> {
        self.0.lock().unwrap_or_else(|p| p.into_inner())
    }
}

pub struct TaskToken<'a> {
    tasks: &'a Tasks,
    id: String,
    flag: Arc<AtomicBool>,
}

impl TaskToken<'_> {
    pub fn flag(&self) -> &AtomicBool {
        &self.flag
    }
}

impl Drop for TaskToken<'_> {
    fn drop(&mut self) {
        self.tasks.lock().remove(&self.id);
    }
}

/// Miroir TypeScript : `TaskProgress` dans `apps/desktop/src/api.ts`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskProgress {
    pub done: u64,
    pub total: u64,
}

/// Envoie la progression au plus toutes les 100 ms (et toujours à la fin),
/// pour ne pas saturer l'interface.
pub struct Throttled<F: FnMut(TaskProgress)> {
    send: F,
    last: Option<Instant>,
}

impl<F: FnMut(TaskProgress)> Throttled<F> {
    pub fn new(send: F) -> Self {
        Self { send, last: None }
    }

    pub fn update(&mut self, done: u64, total: u64) {
        let due = self
            .last
            .is_none_or(|t| t.elapsed() >= Duration::from_millis(100));
        if due || done >= total {
            self.last = Some(Instant::now());
            (self.send)(TaskProgress { done, total });
        }
    }
}

/// Progression vers un canal de l'interface (erreurs d'envoi ignorées :
/// fenêtre fermée pendant la tâche).
pub fn to_channel(channel: Channel<TaskProgress>) -> impl FnMut(TaskProgress) {
    move |p| {
        let _ = channel.send(p);
    }
}

#[tauri::command]
pub fn cancel_task(tasks: tauri::State<'_, Tasks>, task_id: String) -> bool {
    tasks.cancel(&task_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn annulation_et_retrait() {
        let tasks = Tasks::default();
        {
            let t = tasks.start("a".into());
            assert!(!t.flag().load(Ordering::Relaxed));
            assert!(tasks.cancel("a"));
            assert!(t.flag().load(Ordering::Relaxed));
        }
        assert!(!tasks.cancel("a"));
    }

    #[test]
    fn progression_limitee() {
        let mut sent = Vec::new();
        let mut p = Throttled::new(|x| sent.push(x));
        for i in 0..1000 {
            p.update(i, 1000);
        }
        p.update(1000, 1000);
        assert_eq!(sent.first().unwrap().done, 0);
        assert_eq!(sent.last().unwrap().done, 1000);
        assert!(sent.len() < 10);
    }
}

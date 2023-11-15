use ic_cdk::{
    export::candid::{CandidType, Deserialize},
    query, update,
};
use std::cell::RefCell;
use std::collections::HashMap;

/*
Possible statuses ->
Todo,
InProgress,
Done,
Backlog,
*/
#[derive(Clone, Debug, Default, PartialEq, CandidType, Deserialize)]
struct Task {
    pub title: String,
    pub description: String,
    pub status: String,
}

type TasksStore = HashMap<String, Task>;

thread_local! {
    static TASK_STORE: RefCell<TasksStore> = RefCell::new(HashMap::new());
}

#[update]
fn add_task(task: Task) -> String {
    match TASK_STORE.try_with(|task_store| {
        let mut ts = (*task_store).borrow_mut();
        let task_id = format!("task_{}", ts.len() + 1);
        ts.insert(task_id.clone(), task);
        task_id
    }) {
        Ok(task_id) => task_id,
        Err(_e) => "Error adding task".to_string(),
    }
}

#[query]
fn get_tasks() -> Option<Vec<Task>> {
    match TASK_STORE.try_with(|task_store: &RefCell<TasksStore>| {
        let ts = task_store.borrow();
        Some(ts.values().cloned().collect())
    }) {
        Ok(tasks) => tasks,
        Err(_e) => None,
    }
}

#[update]
fn change_task_status(task_id: String, status: String) -> Option<String> {
    match TASK_STORE.try_with(|task_store| {
        let mut ts = (*task_store).borrow_mut();
        if let Some(task) = ts.get_mut(&task_id) {
            task.status = status;
            Some("done.".to_string())
        } else {
            None
        }
    }) {
        Ok(res) => res,
        Err(_e) => None,
    }
}

#[query]
fn get_task_with_pagination(page_no: u128, page_size: u128) -> Option<Vec<Task>> {
    match TASK_STORE.try_with(|task_store: &RefCell<TasksStore>| {
        let ts = task_store.borrow();
        let total_tasks = (ts.len()) as u128;
        let start_index = 1 + ((page_no - 1) * page_size);
        let end_index = start_index + page_size;

        if start_index >= total_tasks {
            // If the start index is beyond the total number of tasks, return None
            None
        } else {
            // Calculate the end index to avoid going beyond the total tasks
            let end_index = end_index.min(total_tasks + 1);
            // Collect tasks with numeric keys within the specified range
            let tasks_slice: Vec<Task> = (start_index..end_index)
                .filter_map(|i| ts.get(&format!("task_{}", i)).cloned())
                .collect();
            Some(tasks_slice)
        }
    }) {
        Ok(tasks) => tasks,
        Err(_e) => None,
    }
}

#[update]
fn update_task_description(task_id: String, description: String) -> Option<String> {
    match TASK_STORE.try_with(|task_store| {
        let mut ts = (*task_store).borrow_mut();
        if let Some(task) = ts.get_mut(&task_id) {
            task.description = description;
            Some("done.".to_string())
        } else {
            None
        }
    }) {
        Ok(res) => res,
        Err(_e) => None,
    }
}

#[update]
fn delete_task_by_id(task_id: String) -> Option<String> {
    match TASK_STORE.try_with(|task_store| {
        let mut ts = (*task_store).borrow_mut();
        if ts.remove(&task_id).is_some() {
            Some("done.".to_string())
        } else {
            None
        }
    }) {
        Ok(res) => res,
        Err(_e) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_task() {
        let task = Task {
            title: "Test Task".to_string(),
            description: "Test Description".to_string(),
            status: "Todo".to_string(),
        };

        let task_id = add_task(task.clone());
        assert_eq!(task_id, "task_1");

        let stored_tasks = TASK_STORE.with(|store| store.borrow().clone());
        assert_eq!(stored_tasks.len(), 1);
        assert_eq!(stored_tasks.get(&task_id).unwrap(), &task);
    }

    #[test]
    fn test_get_tasks() {
        let tasks = vec![
            Task {
                title: "Task 1".to_string(),
                description: "Description 1".to_string(),
                status: "Todo".to_string(),
            },
            Task {
                title: "Task 2".to_string(),
                description: "Description 2".to_string(),
                status: "InProgress".to_string(),
            },
        ];

        TASK_STORE.with(|store| {
            let mut store = store.borrow_mut();
            for (i, task) in tasks.iter().enumerate() {
                store.insert(format!("task_{}", i + 1), task.clone());
            }
        });

        let retrieved_tasks: Vec<Task> = get_tasks().unwrap();
        // assert_eq!(retrieved_tasks, tasks);
        for task in &tasks {
            assert!(
                retrieved_tasks.iter().any(|t| t == task),
                "Task not found: {:?}",
                task
            );
        }
    }

    #[test]
    fn test_change_task_status() {
        let task = Task {
            title: "Test Task".to_string(),
            description: "Test Description".to_string(),
            status: "Todo".to_string(),
        };

        let task_id = add_task(task.clone());

        let result = change_task_status(task_id.clone(), "Done".to_string());
        assert_eq!(result, Some("done.".to_string()));

        let updated_task = TASK_STORE.with(|store| store.borrow().get(&task_id).unwrap().clone());
        assert_eq!(updated_task.status, "Done");
    }

    #[test]
    fn test_get_tasks_with_pagination() {
        let tasks = vec![
            Task {
                title: "Task 1".to_string(),
                description: "Description 1".to_string(),
                status: "Todo".to_string(),
            },
            Task {
                title: "Task 2".to_string(),
                description: "Description 2".to_string(),
                status: "InProgress".to_string(),
            },
            Task {
                title: "Task 3".to_string(),
                description: "Description 3".to_string(),
                status: "Done".to_string(),
            },
            Task {
                title: "Task 4".to_string(),
                description: "Description 4".to_string(),
                status: "Done".to_string(),
            },
        ];

        TASK_STORE.with(|store| {
            let mut store = store.borrow_mut();
            for (i, task) in tasks.iter().enumerate() {
                store.insert(format!("task_{}", i + 1), task.clone());
            }
        });

        let paginated_tasks: Vec<Task> = get_task_with_pagination(1, 2).unwrap();
        assert_eq!(paginated_tasks, vec![tasks[0].clone(), tasks[1].clone()]);
        let paginated_tasks: Vec<Task> = get_task_with_pagination(2, 1).unwrap();
        assert_eq!(paginated_tasks, vec![tasks[1].clone()]);
        let paginated_tasks: Vec<Task> = get_task_with_pagination(2, 2).unwrap();
        assert_eq!(paginated_tasks, vec![tasks[2].clone(), tasks[3].clone()]);
    }

    #[test]
    fn test_update_todo_description() {
        let task = Task {
            title: "Test Task".to_string(),
            description: "Test Description".to_string(),
            status: "Todo".to_string(),
        };

        let task_id = add_task(task.clone());

        let result = update_task_description(task_id.clone(), "Updated Description".to_string());
        assert_eq!(result, Some("done.".to_string()));

        let updated_task = TASK_STORE.with(|store| store.borrow().get(&task_id).unwrap().clone());
        assert_eq!(updated_task.description, "Updated Description");
    }

    #[test]
    fn test_delete_todo_by_id() {
        let task = Task {
            title: "Test Task".to_string(),
            description: "Test Description".to_string(),
            status: "Todo".to_string(),
        };

        let task_id = add_task(task.clone());

        let result = delete_task_by_id(task_id.clone());
        assert_eq!(result, Some("done.".to_string()));

        let stored_tasks = TASK_STORE.with(|store| store.borrow().clone());
        assert_eq!(stored_tasks.get(&task_id), None);
    }
}

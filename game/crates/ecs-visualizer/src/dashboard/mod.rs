use anyhow::{Result, ensure};
use maud::{DOCTYPE, PreEscaped, html};
use serde::Serialize;

use crate::query::Query;

const SCRIPT: &str = include_str!("app.js");
const SOURCE_PATH: &str = "crates/game/src/camera.rs";

const COLUMNS: [Column; 10] = [
    Column::new("Entity", "ID"),
    Column::new("Hero", "H"),
    Column::new("Selected", "Sel"),
    Column::new("Active", "Act"),
    Column::new("TopDownCamera", "Top"),
    Column::new("OverShoulderCamera", "OS"),
    Column::new("Camera", "Cam"),
    Column::new("Transform", "Xf"),
    Column::new("ChildOf", "Par"),
    Column::new("Gamepad", "Pad"),
];

const ENTITIES: [EntityRow; 5] = [
    EntityRow::new(
        "Active hero",
        "Hero / Selected / Active",
        &[
            ("Entity", "12v1"),
            ("Hero", "marker"),
            ("Selected", "true"),
            ("Active", "true"),
            ("Transform", "(0, 0, 0.125)"),
        ],
    ),
    EntityRow::new(
        "Top-down camera",
        "TopDownCamera",
        &[
            ("Entity", "31v1"),
            ("TopDownCamera", "marker"),
            ("Camera", "active=false"),
            ("Transform", "(0, 0, 24)"),
        ],
    ),
    EntityRow::new(
        "Shoulder camera",
        "OverShoulderCamera",
        &[
            ("Entity", "32v1"),
            ("OverShoulderCamera", "marker"),
            ("Camera", "active=true"),
            ("Transform", "(0.225, -0.75, 0.25)"),
            ("ChildOf", "12v1"),
        ],
    ),
    EntityRow::new(
        "Connected gamepad",
        "Gamepad",
        &[("Entity", "7v1"), ("Gamepad", "connected")],
    ),
    EntityRow::new(
        "Boss enemy",
        "Enemy / Boss",
        &[("Entity", "18v1"), ("Transform", "(0.75, 0.5, 0.125)")],
    ),
];

#[derive(Serialize)]
struct DashboardData<'a> {
    source: &'static str,
    queries: &'a [Query],
}

#[derive(Clone, Copy)]
struct Column {
    name: &'static str,
    short: &'static str,
}

impl Column {
    const fn new(name: &'static str, short: &'static str) -> Self {
        Self { name, short }
    }
}

#[derive(Clone, Copy)]
struct EntityRow {
    label: &'static str,
    archetype: &'static str,
    values: &'static [(&'static str, &'static str)],
}

impl EntityRow {
    const fn new(
        label: &'static str,
        archetype: &'static str,
        values: &'static [(&'static str, &'static str)],
    ) -> Self {
        Self {
            label,
            archetype,
            values,
        }
    }

    fn value(self, component: &str) -> Option<&'static str> {
        self.values
            .iter()
            .find_map(|(name, value)| (*name == component).then_some(*value))
    }
}

pub(crate) fn render(queries: &[Query]) -> Result<String> {
    for rule in queries.iter().flat_map(|query| &query.rules) {
        ensure!(
            COLUMNS.iter().any(|column| column.name == rule.component),
            "query component {} has no dashboard column",
            rule.component
        );
    }
    let data = serde_json::to_string(&DashboardData {
        source: SOURCE_PATH,
        queries,
    })?;

    Ok(html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "Arenic ECS Query Visualizer" }
                script src="https://cdn.jsdelivr.net/npm/@tailwindcss/browser@4" {}
                script { (PreEscaped(SCRIPT)) }
                script defer src="https://cdn.jsdelivr.net/npm/alpinejs@3.15.12/dist/cdn.min.js" {}
            }
            body class="min-h-screen bg-slate-50 text-slate-950 antialiased dark:bg-slate-950 dark:text-slate-100" {
                main id="ecs-dashboard" x-data="ecsDashboard" data-dashboard=(data)
                    class="w-full px-3 py-8 sm:px-6 lg:px-8" {
                    header class="mb-7 flex flex-col gap-3 sm:flex-row sm:items-end sm:justify-between" {
                        div {
                            p class="mb-2 font-mono text-xs font-medium uppercase tracking-[0.18em] text-slate-500 dark:text-slate-400" {
                                "Arenic / ECS workbench"
                            }
                            h1 class="text-2xl font-medium tracking-tight sm:text-3xl" {
                                "Camera query sheet"
                            }
                        }
                        p class="font-mono text-xs text-slate-500 dark:text-slate-400" { (SOURCE_PATH) }
                    }

                    section aria-labelledby="query-heading" class="mb-6" {
                        div class="mb-3 flex flex-wrap items-baseline justify-between gap-2" {
                            h2 id="query-heading" class="text-sm font-medium" { "Select a system query" }
                            p class="text-xs text-slate-500 dark:text-slate-400" {
                                "The sheet is a representative runtime snapshot."
                            }
                        }
                        div class="flex flex-wrap gap-2" role="group" aria-label="Camera system queries" {
                            @for (index, query) in queries.iter().enumerate() {
                                button type="button"
                                    x-on:click=(format!("activeQuery = {index}"))
                                    x-bind:data-selected=(format!("activeQuery === {index}"))
                                    x-bind:aria-pressed=(format!("activeQuery === {index}"))
                                    class="rounded-lg border border-slate-300 bg-white px-3 py-2 text-left text-slate-900 transition hover:bg-slate-100 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 focus:ring-offset-slate-50 data-[selected=true]:border-indigo-600 data-[selected=true]:bg-indigo-600 data-[selected=true]:text-white data-[selected=true]:shadow-lg dark:border-slate-700 dark:bg-slate-900 dark:text-slate-100 dark:hover:bg-slate-800 dark:focus:ring-offset-slate-950 dark:data-[selected=true]:border-indigo-400 dark:data-[selected=true]:bg-indigo-500" {
                                    span class="block font-mono text-sm font-medium" { (&query.parameter) }
                                    span class="block text-xs opacity-70" { (&query.system) }
                                }
                            }
                        }
                    }

                    section class="mb-5 grid gap-3 rounded-xl border border-slate-200 bg-white p-4 shadow-sm dark:border-slate-700 dark:bg-slate-900 sm:grid-cols-[minmax(0,1fr)_auto] sm:items-center"
                        aria-live="polite" {
                        div class="min-w-0" {
                            div class="mb-1 flex flex-wrap items-center gap-x-2 gap-y-1" {
                                span x-text="query.system + ' · ' + query.parameter"
                                    class="font-mono text-sm font-medium" {}
                                span x-text="query.kind"
                                    class="rounded-full bg-slate-100 px-2 py-0.5 font-mono text-xs text-slate-600 dark:bg-slate-800 dark:text-slate-300" {}
                            }
                            code x-text="query.signature"
                                class="block break-words text-xs text-slate-500 dark:text-slate-400" {}
                        }
                        div class="flex items-baseline gap-2 sm:justify-end" {
                            span x-text="matchCount" class="text-2xl font-medium tabular-nums" {}
                            span class="text-xs text-slate-500 dark:text-slate-400" { "matching row(s)" }
                        }
                    }

                    section class="overflow-hidden rounded-xl border border-slate-200 bg-white shadow-xl shadow-slate-900/10 dark:border-slate-700 dark:bg-slate-900 dark:shadow-black/30"
                        aria-labelledby="sheet-heading" {
                        h2 id="sheet-heading" class="sr-only" { "Entity component table" }
                        table class="w-full table-fixed border-separate border-spacing-0"
                            aria-describedby="sheet-description" {
                            caption id="sheet-description" class="sr-only" {
                                "Rows are entities and columns are components. Selected query cells rise while unrelated cells fade."
                            }
                            thead {
                                tr class="[&>*:last-child]:border-r-0" {
                                    th class="w-[18%] border-b border-r border-slate-200 bg-slate-100 px-2 py-1 text-left font-mono text-xs font-medium text-slate-500 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-400 sm:w-[15%]"
                                        scope="col" { "ROW" }
                                    @for (index, _) in COLUMNS.iter().enumerate() {
                                        th class="border-b border-r border-slate-200 bg-slate-100 px-1 py-1 text-center font-mono text-xs font-medium text-slate-500 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-400"
                                            scope="col" { ((b'A' + index as u8) as char) }
                                    }
                                }
                                tr class="[&>*:last-child]:border-r-0" {
                                    th class="w-[18%] border-b border-r border-slate-200 bg-slate-50 px-3 py-3 text-left text-xs font-medium dark:border-slate-700 dark:bg-slate-900 sm:w-[15%]"
                                        scope="col" { "Entity archetype" }
                                    @for column in COLUMNS {
                                        th class="border-b border-r border-slate-200 bg-slate-50 px-1 py-3 text-center font-mono text-xs font-medium dark:border-slate-700 dark:bg-slate-900"
                                            scope="col" aria-label=(column.name) {
                                            span class="hidden sm:inline" { (column.name) }
                                            span class="sm:hidden" { (column.short) }
                                        }
                                    }
                                }
                            }
                            tbody id="sheet-body" {
                                @for (row_index, entity) in ENTITIES.iter().copied().enumerate() {
                                    tr class="last:[&>*]:border-b-0 [&>*:last-child]:border-r-0" {
                                        th x-bind:data-matched=(format!("matches({row_index})"))
                                            class="w-[18%] border-b border-r border-slate-200 bg-slate-100 px-3 py-3 text-left transition data-[matched=false]:opacity-30 data-[matched=true]:bg-indigo-50 motion-reduce:transition-none dark:border-slate-700 dark:bg-slate-800 dark:data-[matched=true]:bg-indigo-950/40 sm:w-[15%]"
                                            scope="row" {
                                            span class="flex items-baseline gap-2" {
                                                span class="font-mono text-xs text-slate-500 dark:text-slate-400" { (row_index + 1) }
                                                span class="truncate text-sm font-medium" { (entity.label) }
                                            }
                                            span class="mt-1 block truncate font-mono text-xs text-slate-500 dark:text-slate-400" {
                                                (entity.archetype)
                                            }
                                        }
                                        @for column in COLUMNS {
                                            @let value = entity.value(column.name);
                                            td x-bind:data-state=(format!("cellState({row_index}, '{}')", column.name))
                                                x-bind:data-mode=(format!("cellMode({row_index}, '{}')", column.name))
                                                class="relative border-b border-r border-slate-200 bg-white p-1 align-middle text-slate-900 transition duration-200 data-[state=inactive]:scale-[0.985] data-[state=inactive]:opacity-[0.16] data-[state=active]:z-10 data-[state=active]:-translate-y-1 data-[state=active]:scale-[1.025] data-[state=active]:shadow-xl data-[state=active]:ring-2 data-[mode=read]:bg-indigo-50 data-[mode=read]:ring-indigo-500 data-[mode=write]:bg-rose-50 data-[mode=write]:ring-rose-500 data-[mode=with]:bg-emerald-50 data-[mode=with]:ring-emerald-500 data-[mode=without]:bg-amber-50 data-[mode=without]:ring-amber-500 data-[mode=optional]:bg-violet-50 data-[mode=optional]:ring-violet-500 motion-reduce:transition-none dark:border-slate-700 dark:bg-slate-900 dark:text-slate-100 dark:data-[mode=read]:bg-indigo-950 dark:data-[mode=write]:bg-rose-950 dark:data-[mode=with]:bg-emerald-950 dark:data-[mode=without]:bg-amber-950 dark:data-[mode=optional]:bg-violet-950"
                                                data-component=(column.name)
                                                data-present=(value.is_some())
                                                aria-label=(format!(
                                                    "{}, {}: {}",
                                                    entity.label,
                                                    column.name,
                                                    value.unwrap_or("absent")
                                                )) {
                                                div class="flex min-h-14 items-center justify-center overflow-hidden px-1 text-center font-mono text-xs" {
                                                    @if let Some(value) = value {
                                                        span class="hidden truncate sm:inline" { (value) }
                                                        span class="h-2.5 w-2.5 rounded-full bg-current sm:hidden" {}
                                                    } @else {
                                                        span class="hidden text-slate-400 dark:text-slate-500 sm:inline" { "—" }
                                                    }
                                                }
                                                span x-bind:data-active=(format!("cellActive({row_index}, '{}')", column.name))
                                                    x-bind:data-mode=(format!("cellMode({row_index}, '{}')", column.name))
                                                    x-text=(format!("accessMark('{}')", column.name))
                                                    class="absolute bottom-1 right-1 hidden h-4 w-4 place-items-center rounded-full font-mono text-xs text-white data-[active=true]:grid data-[mode=read]:bg-indigo-600 data-[mode=write]:bg-rose-600 data-[mode=with]:bg-emerald-600 data-[mode=without]:bg-amber-600 data-[mode=optional]:bg-violet-600" {}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    section aria-label="Query access legend"
                        class="mt-5 flex flex-wrap gap-x-5 gap-y-2 text-xs text-slate-500 dark:text-slate-400" {
                        @for (color, label) in [
                            ("bg-indigo-600", "Read"),
                            ("bg-rose-600", "Mutable"),
                            ("bg-emerald-600", "With filter"),
                            ("bg-amber-600", "Without filter"),
                            ("bg-violet-600", "Optional read"),
                        ] {
                            span class="inline-flex items-center gap-2" {
                                span class=(format!("h-2.5 w-2.5 rounded-full {color}")) {}
                                (label)
                            }
                        }
                    }
                }
            }
        }
    }
    .into_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_camera_queries_with_tailwind_and_alpine() {
        let queries = crate::query::extract(include_str!("../../../game/src/camera.rs"))
            .expect("camera queries should parse");
        let html = render(&queries).expect("dashboard should render");

        assert_eq!(html.matches("<!DOCTYPE html>").count(), 1);
        assert!(html.contains("@tailwindcss/browser@4"));
        assert!(html.contains("alpinejs@3.15.12"));
        assert!(html.contains("attach_over_shoulder_camera"));
        assert!(html.contains("x-data=\"ecsDashboard\""));
        assert!(!html.contains("<style"));
    }
}

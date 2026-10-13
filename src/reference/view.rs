//! Справка по Anime4K: секции интерфейса.

use egui::{Color32, RichText};

use super::data::{MODES, PIPELINES, SHADERS};
use super::widgets::{
    accent, badge, bullets, card, card_fill, mode_color, note_fill, para, section,
};

/// Рисует содержимое справки. `russian` — язык интерфейса.
pub fn show(ui: &mut egui::Ui, russian: bool) {
    show_header(ui, russian);
    show_picker(ui, russian);
    show_modes(ui, russian);
    show_advanced(ui, russian);
    show_tips(ui, russian);
}

fn show_header(ui: &mut egui::Ui, russian: bool) {
    // --- шапка ---
    let title = if russian {
        "Инструкция по Anime4K"
    } else {
        "Anime4K usage guide"
    };
    let subtitle = if russian {
        "Три основных режима + дополнительные. Правильный режим — тот, что выглядит лучше всего."
    } else {
        "Three major modes + secondary ones. The correct mode is the one that looks best."
    };
    card(ui, card_fill(ui), |ui| {
        ui.label(RichText::new(title).size(18.0).strong());
        ui.label(
            RichText::new(subtitle)
                .size(13.0)
                .italics()
                .color(ui.visuals().weak_text_color()),
        );
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            for id in ["A", "B", "C", "A+A", "B+B", "C+A"] {
                ui.label(badge(id, mode_color(id)));
            }
        });
    });
}

fn show_picker(ui: &mut egui::Ui, russian: bool) {
    // --- как выбрать ---
    section(
        ui,
        if russian {
            "1 · Как выбрать режим"
        } else {
            "1 · How to pick a mode"
        },
    );
    para(
        ui,
        if russian {
            "Небольшие CNN-сети не покрывают все типы деградации из реального мира, поэтому выбор остаётся за глазами зрителя."
        } else {
            "Small CNN networks cannot cover every real-world degradation, so human judgement picks the winner."
        },
    );
    for (i, step) in if russian {
        [
            "Посмотрите каждый режим по порядку A → B → C.",
            "Режим A при неверном выборе даёт самые заметные артефакты; B и C на низком разрешении различить сложнее.",
            "Для роста качества восприятия берите соответствующий дополнительный режим.",
        ]
        .iter()
    } else {
        [
            "Preview each mode in A → B → C order.",
            "Mode A shows the most visible artifacts when wrong; B and C are harder apart on low resolution.",
            "For extra perceptual quality take the matching secondary mode.",
        ]
        .iter()
    }
    .enumerate()
    {
        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new(format!("{}.", i + 1))
                    .strong()
                    .color(accent(ui)),
            );
            ui.label(RichText::new(*step).size(13.5));
        });
    }
    ui.add_space(4.0);

    // --- таблица соответствия ---
    ui.label(
        RichText::new(if russian {
            "Основной → дополнительный"
        } else {
            "Primary → secondary"
        })
        .strong()
        .size(13.5),
    );
    ui.add_space(2.0);
    egui::Grid::new("ref_secondary")
        .striped(true)
        .num_columns(3)
        .spacing([12.0, 4.0])
        .show(ui, |ui| {
            ui.label(
                RichText::new(if russian {
                    "Основной"
                } else {
                    "Primary"
                })
                .strong()
                .size(12.5),
            );
            ui.label(RichText::new("").size(12.5));
            ui.label(
                RichText::new(if russian {
                    "Дополнительный"
                } else {
                    "Secondary"
                })
                .strong()
                .size(12.5),
            );
            ui.end_row();
            for (a, b) in [("A", "A+A"), ("B", "B+B"), ("C", "C+A")] {
                ui.label(badge(a, mode_color(a)));
                ui.label(RichText::new("→").strong().color(accent(ui)));
                ui.label(badge(b, mode_color(b)));
                ui.end_row();
            }
        });
    ui.add_space(4.0);
}

fn show_modes(ui: &mut egui::Ui, russian: bool) {
    // --- карточки режимов ---
    section(
        ui,
        if russian {
            "2 · Что делает каждый режим"
        } else {
            "2 · What each mode does"
        },
    );
    let ok = Color32::from_rgb(46, 125, 50);
    let warn = Color32::from_rgb(198, 40, 40);
    for mode in MODES {
        let open_default = matches!(mode.id, "A" | "B" | "C");
        egui::CollapsingHeader::new(badge(mode.id, mode_color(mode.id)))
            .default_open(open_default)
            .show(ui, |ui| {
                ui.label(
                    RichText::new(if russian { mode.for_ru } else { mode.for_en })
                        .size(13.0)
                        .italics(),
                );
                ui.add_space(2.0);
                ui.label(
                    RichText::new(if russian { "Плюсы" } else { "Pros" })
                        .strong()
                        .size(12.5)
                        .color(ok),
                );
                bullets(ui, if russian { mode.plus_ru } else { mode.plus_en });
                ui.label(
                    RichText::new(if russian {
                        "Минусы (при неверном выборе)"
                    } else {
                        "Cons (if misused)"
                    })
                    .strong()
                    .size(12.5)
                    .color(warn),
                );
                bullets(
                    ui,
                    if russian {
                        mode.minus_ru
                    } else {
                        mode.minus_en
                    },
                );
            });
    }

    // --- важное замечание ---
    card(ui, note_fill(ui), |ui| {
        ui.label(
            RichText::new(if russian { "Важно" } else { "Important" })
                .strong()
                .size(13.5),
        );
        para(
            ui,
            if russian {
                "Режимы A+A / B+B / C+A — только для масштабирования x2 и выше. На экране 1080p обычный A для аниме 1080p улучшит картинку, а A+A скорее даст перешарп и ухудшит её."
            } else {
                "A+A / B+B / C+A are for upscaling ratios x2 and above only. On a 1080p screen plain A improves 1080p anime, while A+A will likely oversharpen and degrade it."
            },
        );
    });
}

fn show_advanced(ui: &mut egui::Ui, russian: bool) {
    // --- расширенная настройка ---
    section(
        ui,
        if russian {
            "3 · Расширенная настройка"
        } else {
            "3 · Advanced customization"
        },
    );
    para(
        ui,
        if russian {
            "Мало пресетов? Интересны странные связки вроде B+A или A+B? Соберите свой конвейер восстановления."
        } else {
            "Want more than presets? Curious about odd combos like B+A or A+B? Build your own restoration pipeline."
        },
    );
    ui.label(
        RichText::new(if russian { "Основы" } else { "Basics" })
            .strong()
            .size(13.0),
    );
    bullets(
        ui,
        if russian {
            &[
                "Каждый шейдер работает отдельно или в связке с любыми другими.",
                "Каждый файл — только один раз. Дубли дают баги и потерю скорости: берите другой вариант либо копируйте и переименовывайте файл.",
                "Порядок обработки = порядок имён файлов в конфиге. Исключение — Clamp_Highlights.",
                "Вариант CNN (S, M, L, VL, UL): каждый шаг размера удваивает время. M за 5 мс → L за ~10 мс → VL за ~20 мс.",
                "Шейдеры без CNN намного быстрее, но качество может быть ниже.",
            ]
        } else {
            &[
                "Every shader works standalone or combined with any others.",
                "Each file only once. Duplicates cause bugs and lost performance: pick another variant or copy and rename.",
                "Processing order = filename order in the config. Exception: Clamp_Highlights.",
                "CNN variant (S, M, L, VL, UL): each size step doubles the cost. M at 5 ms → L at ~10 ms → VL at ~20 ms.",
                "Non-CNN shaders are much faster but may look worse.",
            ]
        },
    );

    // --- типы шейдеров ---
    ui.label(
        RichText::new(if russian {
            "Типы шейдеров"
        } else {
            "Shader types"
        })
        .strong()
        .size(13.0),
    );
    ui.add_space(2.0);
    egui::Grid::new("ref_shaders")
        .striped(true)
        .num_columns(2)
        .spacing([10.0, 6.0])
        .show(ui, |ui| {
            for s in SHADERS {
                ui.label(RichText::new(s.name).monospace().size(12.5).strong());
                ui.add(
                    egui::Label::new(RichText::new(if russian { s.ru } else { s.en }).size(12.5))
                        .wrap(),
                );
                ui.end_row();
            }
        });
    ui.add_space(4.0);

    // --- состав режимов ---
    ui.label(
        RichText::new(if russian {
            "Состав режимов по умолчанию"
        } else {
            "Default mode pipelines"
        })
        .strong()
        .size(13.0),
    );
    ui.add_space(2.0);
    egui::Grid::new("ref_pipelines")
        .striped(true)
        .num_columns(2)
        .spacing([12.0, 4.0])
        .show(ui, |ui| {
            for (id, pipe) in PIPELINES {
                ui.label(badge(id, mode_color(id)));
                ui.label(RichText::new(*pipe).monospace().size(12.5));
                ui.end_row();
            }
        });
    para(
        ui,
        if russian {
            "Clamp_Highlights и AutoDownscalePre убраны из таблицы для наглядности."
        } else {
            "Clamp_Highlights and AutoDownscalePre omitted for clarity."
        },
    );

    // --- как определяются режимы ---
    ui.label(
        RichText::new(if russian {
            "Как определяются режимы"
        } else {
            "How modes are defined"
        })
        .strong()
        .size(13.0),
    );
    bullets(
        ui,
        if russian {
            &[
                "A изначально: Restore → Upscale.",
                "B изначально: Restore_Soft → Upscale.",
                "C изначально: Upscale.",
                "Без стартового Restore нужен Upscale_Denoise или Denoise — почти всё видеосжатие с потерями.",
                "Добавляйте апскейл, пока конвейер не даст минимум 4x (480p → 4K ≈ 4.5x).",
            ]
        } else {
            &[
                "A is defined as: Restore → Upscale.",
                "B is defined as: Restore_Soft → Upscale.",
                "C is defined as: Upscale.",
                "Without a leading Restore, start with Upscale_Denoise or Denoise — video compression is lossy.",
                "Add upscalers until the pipeline reaches at least 4x (480p → 4K ≈ 4.5x).",
            ]
        },
    );

    // --- пример ---
    card(ui, card_fill(ui), |ui| {
        ui.label(
            RichText::new(if russian {
                "Пример: что такое C+A+B"
            } else {
                "Example: what C+A+B is"
            })
            .strong()
            .size(13.0),
        );
        ui.monospace("Upscale_Denoise → Restore → Upscale → Restore_Soft → Upscale");
        para(
            ui,
            if russian {
                "1) C (Upscale) → A (Restore → Upscale) → B (Restore_Soft → Upscale). 2) Начало без Restore заменяем на Upscale_Denoise. 3) Коэффициент 4x уже набран. Варианты S/M/L/VL/UL — на выбор."
            } else {
                "1) C (Upscale) → A (Restore → Upscale) → B (Restore_Soft → Upscale). 2) Leading non-Restore becomes Upscale_Denoise. 3) 4x ratio already met. Pick S/M/L/VL/UL variants at will."
            },
        );
    });
}

fn show_tips(ui: &mut egui::Ui, russian: bool) {
    // --- рекомендации ---
    section(
        ui,
        if russian {
            "4 · Рекомендации"
        } else {
            "4 · Best practices"
        },
    );
    bullets(
        ui,
        if russian {
            &[
                "Всегда ставьте Clamp_Highlights в начало против звона; без него чуть быстрее.",
                "Restore после апскейла улучшает восприятие, но тормозит и может дать артефакты.",
                "Шейдеры после шага x2 стоят в 4 раза дороже: 10 мс до апскейла → 40 мс после. Компенсируйте вариантом CNN на два шага ниже (S вместо L).",
                "Артефакты слабых вариантов (M/S) незаметны на высоком разрешении — так снижают шум, нагрев и потребление GPU.",
                "Цель для 24 fps — около 41 мс. При пропусках кадров берите варианты быстрее. Проверка — профайлер mpv (Shift+I, затем 2).",
            ]
        } else {
            &[
                "Always put Clamp_Highlights first against ringing; dropping it is slightly faster.",
                "Restore after upscaling helps perception but is slower and may add artifacts.",
                "Shaders after a x2 step cost 4x: 10 ms before → 40 ms after. Compensate with a CNN variant two steps below (S instead of L).",
                "Weak-variant (M/S) artifacts are invisible at high resolution — use that to cut GPU noise, heat and power.",
                "Target for 24 fps is ~41 ms. On frame drops pick faster variants. Check with the mpv profiler (Shift+I, then 2).",
            ]
        },
    );
    egui::Grid::new("ref_fps")
        .striped(true)
        .num_columns(2)
        .spacing([12.0, 4.0])
        .show(ui, |ui| {
            ui.label(
                RichText::new(if russian {
                    "Частота видео"
                } else {
                    "Video framerate"
                })
                .strong()
                .size(12.5),
            );
            ui.label(
                RichText::new(if russian {
                    "Лимит (мс)"
                } else {
                    "Budget (ms)"
                })
                .strong()
                .size(12.5),
            );
            ui.end_row();
            for (fps, ms) in [("24", "41"), ("30", "33"), ("60", "16")] {
                ui.monospace(fps);
                ui.monospace(ms);
                ui.end_row();
            }
        });
    ui.add_space(8.0);
}

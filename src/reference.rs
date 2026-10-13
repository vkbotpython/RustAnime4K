//! Красивая справка по Anime4K (замена моноширинной простыни).
//!
//! Рендерит структурированный документ из `assets/docs/GLSL_Instructions_*`:
//! заголовок, шаги выбора режима, таблицы, карточки режимов, типы шейдеров.

use egui::{Color32, RichText, Stroke};

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

fn accent(ui: &egui::Ui) -> Color32 {
    ui.visuals().hyperlink_color
}

fn section(ui: &mut egui::Ui, title: &str) {
    ui.add_space(10.0);
    ui.label(RichText::new(title).size(15.0).strong().color(accent(ui)));
    ui.separator();
    ui.add_space(2.0);
}

fn para(ui: &mut egui::Ui, text: &str) {
    ui.add(egui::Label::new(RichText::new(text).size(13.5)).wrap());
    ui.add_space(4.0);
}

fn bullets(ui: &mut egui::Ui, items: &[&str]) {
    for item in items {
        ui.add(egui::Label::new(RichText::new(format!("• {item}")).size(13.0)).wrap());
    }
    ui.add_space(4.0);
}

fn badge(text: &str, bg: Color32) -> RichText {
    RichText::new(format!(" {text} "))
        .strong()
        .size(13.0)
        .color(Color32::WHITE)
        .background_color(bg)
}

fn mode_color(id: &str) -> Color32 {
    match id {
        "A" => Color32::from_rgb(211, 47, 47),
        "B" => Color32::from_rgb(25, 118, 210),
        "C" => Color32::from_rgb(56, 142, 60),
        "A+A" => Color32::from_rgb(123, 31, 162),
        "B+B" => Color32::from_rgb(0, 121, 107),
        "C+A" => Color32::from_rgb(230, 119, 0),
        _ => Color32::from_rgb(90, 90, 90),
    }
}

fn card(ui: &mut egui::Ui, fill: Color32, add: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(fill)
        .inner_margin(10.0)
        .corner_radius(8.0)
        .stroke(Stroke::new(
            1.0,
            ui.visuals().widgets.noninteractive.bg_stroke.color,
        ))
        .show(ui, add);
    ui.add_space(6.0);
}

fn card_fill(ui: &egui::Ui) -> Color32 {
    // Лёгкая подложка поверх текущей темы (работает и в тёмной, и в светлой).
    let base = ui.visuals().faint_bg_color;
    // `faint_bg_color` уже полупрозрачный — делаем чуть плотнее для карточки.
    Color32::from_rgba_unmultiplied(base.r(), base.g(), base.b(), 220)
}

fn note_fill(ui: &egui::Ui) -> Color32 {
    if ui.visuals().dark_mode {
        Color32::from_rgb(64, 50, 12)
    } else {
        Color32::from_rgb(255, 248, 225)
    }
}

// ---------------------------------------------------------------------------
// data
// ---------------------------------------------------------------------------

struct ModeInfo {
    id: &'static str,
    for_ru: &'static str,
    for_en: &'static str,
    plus_ru: &'static [&'static str],
    plus_en: &'static [&'static str],
    minus_ru: &'static [&'static str],
    minus_en: &'static [&'static str],
}

const MODES: &[ModeInfo] = &[
    ModeInfo {
        id: "A",
        for_ru: "Большинство аниме 1080p. Некоторые старые 720p. Большинство старых SD. Сильное размытие, много артефактов передискретизации, размазывание от сжатия.",
        for_en: "Most 1080p anime. Some older 720p. Most old SD anime. Heavy blur, lots of resampling artifacts, smearing from compression.",
        plus_ru: &[
            "Высокое качество восприятия",
            "Уменьшает артефакты сжатия",
            "Восстанавливает наиболее повреждённые линии",
            "Уменьшает сильное размытие и шум",
        ],
        plus_en: &[
            "High perceptual quality",
            "Reduces compression artifacts",
            "Reconstructs most degraded lines",
            "Reduces heavy blur and noise",
        ],
        minus_ru: &[
            "Может усиливать звон, если он уже есть",
            "Может усиливать полосатость (banding)",
            "Сильное шумоподавление может размыть текстуры",
        ],
        minus_en: &[
            "Can amplify ringing if already present",
            "Can amplify banding if already present",
            "Strong denoising may blur textures",
        ],
    },
    ModeInfo {
        id: "B",
        for_ru: "Некоторые 1080p. Большинство 720p. Даунскейл 1080p → 720p. Слабое размытие, лёгкие артефакты ресемплинга, звон от даунскейла.",
        for_en: "Some 1080p. Most 720p. 1080p → 720p downscaled anime. Light blur, some resampling artifacts, ringing from downsampling.",
        plus_ru: &[
            "Уменьшает артефакты сжатия",
            "Восстанавливает часть повреждённых линий",
            "Уменьшает размытие, шум, звон и алиасинг",
        ],
        plus_en: &[
            "Reduces compression artifacts",
            "Reconstructs some degraded lines",
            "Reduces blur, noise, ringing and aliasing",
        ],
        minus_ru: &[
            "Часть артефактов может остаться",
            "Часть линий может остаться размытой",
            "Сильное шумоподавление может размыть текстуры",
        ],
        minus_en: &[
            "Some artifacts may remain",
            "Some lines may stay blurry",
            "Strong denoising may blur textures",
        ],
    },
    ModeInfo {
        id: "C",
        for_ru: "Даунскейл 1080p → 480p. Редко — 1080p полнометражки. Чистые изображения без деградации, обои, Pixiv-арт.",
        for_en: "1080p → 480p downscaled anime. Rarely 1080p movies. Clean images with no degradation, wallpapers, Pixiv art.",
        plus_ru: &["Самый высокий PSNR", "Снижает шум"],
        plus_en: &["Highest PSNR", "Reduces noise"],
        minus_ru: &[
            "Низкое качество восприятия",
            "Может усиливать звон и артефакты ресемплинга",
        ],
        minus_en: &[
            "Low perceptual quality",
            "Can amplify ringing and resampling artifacts",
        ],
    },
    ModeInfo {
        id: "A+A",
        for_ru: "То же, что и A. Только для апскейла x2 и выше.",
        for_en: "Same as A. Only for upscaling ratios x2 and above.",
        plus_ru: &[
            "Высочайшее качество восприятия",
            "Восстанавливает почти все повреждённые линии",
        ],
        plus_en: &[
            "Highest perceptual quality",
            "Reconstructs almost all degraded lines",
        ],
        minus_ru: &[
            "Может дать сильный звон, полосатость, алиасинг",
            "Медленнее, чем обычный A",
        ],
        minus_en: &[
            "Can cause severe ringing, banding, aliasing",
            "Slower than plain A",
        ],
    },
    ModeInfo {
        id: "B+B",
        for_ru: "То же, что и B. Только для апскейла x2 и выше.",
        for_en: "Same as B. Only for upscaling ratios x2 and above.",
        plus_ru: &["Высокое качество восприятия, плюсы режима B"],
        plus_en: &["High perceptual quality, same pros as B"],
        minus_ru: &["Те же минусы, что у B (слабее выражены)", "Медленнее, чем обычный B"],
        minus_en: &["Same cons as B (milder)", "Slower than plain B"],
    },
    ModeInfo {
        id: "C+A",
        for_ru: "То же, что и C. Только для апскейла x2 и выше.",
        for_en: "Same as C. Only for upscaling ratios x2 and above.",
        plus_ru: &["Чуть выше качество восприятия, чем у C"],
        plus_en: &["Slightly higher perceptual quality than C"],
        minus_ru: &["Те же минусы, что у C (слабее выражены)", "Медленнее, чем обычный C"],
        minus_en: &["Same cons as C (milder)", "Slower than plain C"],
    },
];

struct ShaderInfo {
    name: &'static str,
    ru: &'static str,
    en: &'static str,
}

const SHADERS: &[ShaderInfo] = &[
    ShaderInfo { name: "Restore", ru: "Фишка Anime4K. Восстанавливает картинку до апскейла: сжатие, размытие, звон. Restore — против апскейл-артефактов и блюра, Restore_Soft — против даунскейла и алиасинга.", en: "What makes Anime4K special. Restores before upscaling: compression, blur, ringing. Restore targets upsampling blur, Restore_Soft targets downsampling and aliasing." },
    ShaderInfo { name: "Upscale", ru: "Масштабирует в 2 раза, считая картинку чистой.", en: "Upscales x2, assuming a clean image." },
    ShaderInfo { name: "Upscale_Denoise", ru: "Масштабирует в 2 раза и давит шум без потери скорости GPU.", en: "Upscales x2 and denoises with no GPU penalty." },
    ShaderInfo { name: "Clamp_Highlights", ru: "Считает статистику в точке подключения и в конце режет света — меньше перерегулирования и звона.", en: "Saves stats where placed, clamps highlights at the end — less overshoot and ringing." },
    ShaderInfo { name: "Darken", ru: "Затемняет линии. Понятие «линия» неоднозначно — на ваш вкус.", en: "Darkens lines. Ambiguous what a line is — use to taste." },
    ShaderInfo { name: "Thin", ru: "Делает линии тоньше. Аналогично — на ваш вкус.", en: "Thins lines. Same caveat — use to taste." },
    ShaderInfo { name: "Denoise", ru: "Обычный фильтр шумоподавления.", en: "Plain denoising filter." },
    ShaderInfo { name: "Deblur", ru: "Повышает резкость деталей без перерегулирования и звона.", en: "Sharpens details without overshoot or ringing." },
    ShaderInfo { name: "AutoDownscalePre_x4", ru: "Даунскейл после первого апскейла, чтобы второй шаг x2 точно попал в экран. Между двумя Upscale.", en: "Downscales after the first upscale so the second x2 step matches the screen. Between two Upscales." },
    ShaderInfo { name: "AutoDownscalePre_x2", ru: "Даунскейл после первого апскейла до размера экрана. После первого Upscale.", en: "Downscales after the first upscale to screen size. After the first Upscale." },
];

const PIPELINES: &[(&str, &str)] = &[
    ("A", "Restore → Upscale → Upscale"),
    ("B", "Restore_Soft → Upscale → Upscale"),
    ("C", "Upscale_Denoise → Upscale"),
    ("A+A", "Restore → Upscale → Restore → Upscale"),
    ("B+B", "Restore_Soft → Upscale → Restore_Soft → Upscale"),
    ("C+A", "Upscale_Denoise → Restore → Upscale"),
];

// ---------------------------------------------------------------------------
// entry
// ---------------------------------------------------------------------------

/// Рисует содержимое справки. `russian` — язык интерфейса.
pub fn show(ui: &mut egui::Ui, russian: bool) {
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

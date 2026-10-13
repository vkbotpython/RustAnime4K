//! Данные справки: режимы, шейдеры, конвейеры.

pub(crate) struct ModeInfo {
    pub(crate) id: &'static str,
    pub(crate) for_ru: &'static str,
    pub(crate) for_en: &'static str,
    pub(crate) plus_ru: &'static [&'static str],
    pub(crate) plus_en: &'static [&'static str],
    pub(crate) minus_ru: &'static [&'static str],
    pub(crate) minus_en: &'static [&'static str],
}

pub(crate) const MODES: &[ModeInfo] = &[
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
        minus_ru: &[
            "Те же минусы, что у B (слабее выражены)",
            "Медленнее, чем обычный B",
        ],
        minus_en: &["Same cons as B (milder)", "Slower than plain B"],
    },
    ModeInfo {
        id: "C+A",
        for_ru: "То же, что и C. Только для апскейла x2 и выше.",
        for_en: "Same as C. Only for upscaling ratios x2 and above.",
        plus_ru: &["Чуть выше качество восприятия, чем у C"],
        plus_en: &["Slightly higher perceptual quality than C"],
        minus_ru: &[
            "Те же минусы, что у C (слабее выражены)",
            "Медленнее, чем обычный C",
        ],
        minus_en: &["Same cons as C (milder)", "Slower than plain C"],
    },
];

pub(crate) struct ShaderInfo {
    pub(crate) name: &'static str,
    pub(crate) ru: &'static str,
    pub(crate) en: &'static str,
}

pub(crate) const SHADERS: &[ShaderInfo] = &[
    ShaderInfo {
        name: "Restore",
        ru: "Фишка Anime4K. Восстанавливает картинку до апскейла: сжатие, размытие, звон. Restore — против апскейл-артефактов и блюра, Restore_Soft — против даунскейла и алиасинга.",
        en: "What makes Anime4K special. Restores before upscaling: compression, blur, ringing. Restore targets upsampling blur, Restore_Soft targets downsampling and aliasing.",
    },
    ShaderInfo {
        name: "Upscale",
        ru: "Масштабирует в 2 раза, считая картинку чистой.",
        en: "Upscales x2, assuming a clean image.",
    },
    ShaderInfo {
        name: "Upscale_Denoise",
        ru: "Масштабирует в 2 раза и давит шум без потери скорости GPU.",
        en: "Upscales x2 and denoises with no GPU penalty.",
    },
    ShaderInfo {
        name: "Clamp_Highlights",
        ru: "Считает статистику в точке подключения и в конце режет света — меньше перерегулирования и звона.",
        en: "Saves stats where placed, clamps highlights at the end — less overshoot and ringing.",
    },
    ShaderInfo {
        name: "Darken",
        ru: "Затемняет линии. Понятие «линия» неоднозначно — на ваш вкус.",
        en: "Darkens lines. Ambiguous what a line is — use to taste.",
    },
    ShaderInfo {
        name: "Thin",
        ru: "Делает линии тоньше. Аналогично — на ваш вкус.",
        en: "Thins lines. Same caveat — use to taste.",
    },
    ShaderInfo {
        name: "Denoise",
        ru: "Обычный фильтр шумоподавления.",
        en: "Plain denoising filter.",
    },
    ShaderInfo {
        name: "Deblur",
        ru: "Повышает резкость деталей без перерегулирования и звона.",
        en: "Sharpens details without overshoot or ringing.",
    },
    ShaderInfo {
        name: "AutoDownscalePre_x4",
        ru: "Даунскейл после первого апскейла, чтобы второй шаг x2 точно попал в экран. Между двумя Upscale.",
        en: "Downscales after the first upscale so the second x2 step matches the screen. Between two Upscales.",
    },
    ShaderInfo {
        name: "AutoDownscalePre_x2",
        ru: "Даунскейл после первого апскейла до размера экрана. После первого Upscale.",
        en: "Downscales after the first upscale to screen size. After the first Upscale.",
    },
];

pub(crate) const PIPELINES: &[(&str, &str)] = &[
    ("A", "Restore → Upscale → Upscale"),
    ("B", "Restore_Soft → Upscale → Upscale"),
    ("C", "Upscale_Denoise → Upscale"),
    ("A+A", "Restore → Upscale → Restore → Upscale"),
    ("B+B", "Restore_Soft → Upscale → Restore_Soft → Upscale"),
    ("C+A", "Upscale_Denoise → Restore → Upscale"),
];

// ---------------------------------------------------------------------------
// entry

#[cfg(feature = "project-analysis")]
use crate::structs::FtaQlConfigResolved;
use crate::structs::HalsteadMetrics;
#[cfg(feature = "project-analysis")]
use ignore::DirEntry;
use log::warn;
use std::path::Path;

// Удаляю все вызовы is_excluded_filename (функция удалена)

#[cfg(feature = "project-analysis")]
pub fn is_valid_file(_repo_path: &String, entry: &DirEntry, _config: &FtaQlConfigResolved) -> bool {
    entry.file_type().map_or(false, |ft| ft.is_file())
}

pub fn warn_about_language(file_name: &str, use_tsx: bool) {
    let tsx_name = if use_tsx { "j/tsx" } else { "non-j/tsx" };
    let opposite_tsx_name = if use_tsx { "non-j/tsx" } else { "j/tsx" };

    warn!(
        "File {} was interpreted as {} but seems to actually be {}. The file extension may be incorrect.",
        file_name,
        tsx_name,
        opposite_tsx_name
    );
}

pub fn check_score_cap_breach(file_name: String, file_score: f64, score_cap: usize) {
    // Exit 1 if score_cap breached
    if file_score > score_cap as f64 {
        eprintln!(
            "File {} has a score of {}, which is beyond the score cap of {}, exiting.",
            file_name, file_score, score_cap
        );
        std::process::exit(1);
    }
}

/// Входные данные для расчёта File Score (расширяемая структура).
#[derive(Debug, Clone)]
pub struct ModuleScoreInput<'a> {
    pub cyclomatic: usize,
    pub halstead: &'a HalsteadMetrics,
    pub line_count: usize,
    // В будущем: coupling, size, и др.
}

/// Перечисление для выбора формулы расчёта File Score.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileScoreFormula {
    /// Оригинальная формула (codehawk-cli)
    Original,
    // В будущем можно добавить другие варианты, например: Experimental, Custom и т.д.
}

/// Вычисляет File Score по переданным метрикам и выбранной формуле.
/// Формула по умолчанию — Original (codehawk-cli).
pub fn calculate_file_score(input: &ModuleScoreInput, formula: FileScoreFormula) -> f64 {
    match formula {
        FileScoreFormula::Original => {
            let line_count_float = input.line_count as f64;
            let program_length_float = input.halstead.program_length as f64;
            let cyclo_float = input.cyclomatic as f64;
            let vocab_float = input.halstead.vocabulary_size as f64;
            let difficulty_float = input.halstead.difficulty as f64;

            const MINIMUM_CYCLO: f64 = 1.0;

            let factor = if cyclo_float.ln() < MINIMUM_CYCLO {
                MINIMUM_CYCLO
            } else {
                let div = if program_length_float > 0.0 {
                    ((cyclo_float.ln() * difficulty_float.ln()).sqrt() * program_length_float.ln())
                        .sqrt()
                } else {
                    1.0
                };
                (line_count_float * difficulty_float).sqrt() / div
            };

            // Нормализационная формула из codehawk-cli
            let absolute_file_score =
                171.0 - 5.2 * vocab_float.ln() - 0.23 * cyclo_float - 16.2 * factor.ln();
            let mut file_score = 100.0 - ((absolute_file_score * 100.0) / 171.0);

            if file_score < 0.0 {
                file_score = 0.0;
            }

            file_score
        } // В будущем: другие формулы
    }
}

/// Перечисление для выбора формулы расчёта Coupling Score.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CouplingScoreFormula {
    /// Оригинальная формула (заглушка)
    Original,
    // В будущем: другие варианты
}

/// Вычисляет Coupling Score по переданным метрикам и выбранной формуле.
/// Пока реализована только Original (заглушка).
pub fn calculate_coupling_score(
    coupling_metrics: &crate::structs::CouplingMetrics,
    all_cycles: &[crate::structs::CycleInfo],
    formula: CouplingScoreFormula,
) -> f64 {
    match formula {
        CouplingScoreFormula::Original => {
            const ALPHA_DS: f64 = 10.0; // «бесплатный» бюджет Ds (сдвиг вправо)
            const K_SLOPE: f64 = 12.0; // крутизна влияния средней силы зависимостей
            const K_CYCLE: f64 = 90.0; // штраф за циклы

            // ---------- 1. Предварительные величины ----------
            let ce = coupling_metrics.efferent_coupling as f64;
            let ca = coupling_metrics.afferent_coupling as f64;
            let c_tot = ce + ca;

            // Если нет связей, то возвращаем 0
            if c_tot == 0.0 {
                return 0.0;
            }

            #[inline]
            fn logistic(x: f64, lambda: f64, tau: f64) -> f64 {
                let top = (lambda * (x - tau)).exp() - 1.0;
                let norm = (lambda * (1.0 - tau)).exp() - 1.0;
                (top / norm).clamp(0.0, 1.0)
            }

            // Instability (доверяем, что пришёл корректный 0..=1)
            let instability = coupling_metrics.instability;

            // --- константы для нестабильности -----------------------------------------
            const K_INSTAB: f64 = 100.0; // максимальный «штрафовой» вес
            const LAMBDA_INSTAB: f64 = 10.0; // крутизна логистической кривой
            const TAU_INSTAB: f64 = 0.8; // «коленка» (Is, после которой штраф растёт)
            const STAB_BONUS: f64 = 30.0; // линейный бонус за стабильность (1-Is)

            /// penalty_instability() возвращает суммы **двух** эффектов:
            ///   1. нелинейного штрафа, резко ускоряющегося при Is > τ;
            ///   2. прежнего «бонуса» за стабильность (линейно от 0 до STAB_BONUS).
            fn penalty_instability(is: f64) -> f64 {
                // (1) нелинейный штраф: 0 … K_INSTAB
                let risk = logistic(is, LAMBDA_INSTAB, TAU_INSTAB); // 0…1
                let non_linear_penalty = K_INSTAB * risk;
                // (2) линейный бонус за стабильность (добавляем к штрафу меньше, если Is меньше)
                let stability_bonus = STAB_BONUS * is;
                non_linear_penalty + stability_bonus
            }

            const K_KEY: f64 = 50.0; // наказание: «нестабилен и нужен»
            const K_ORPH: f64 = 50.0; // наказание: «стабилен и не нужен»
            const LAMBDA_KEY: f64 = 10.0; // крутизна логистической кривой
            const TAU_KEY: f64 = 0.8; // порог Is для key_risk
            const LAMBDA_ORPH: f64 = 10.0; // крутизна логистической кривой
            const TAU_ORPH: f64 = 0.2; // порог Is для orphan_risk

            /// 1) «Плохое ядро»: высокие Ca и Is
            fn key_risk(ca: f64, is: f64) -> f64 {
                let keyness = ca / (ca + 1.0);
                let instab = logistic(is, LAMBDA_KEY, TAU_KEY);
                keyness * instab
            }

            /// 2) «Лишняя абстракция»: низкие Ca и Is
            fn orphan_risk(ca: f64, is: f64) -> f64 {
                // 1) «невостребованность» по Ca
                let lack_of_use = if ca <= 1.0 {
                    1.0 // полный эффект при 0…1 входящих
                } else {
                    const LAMBDA_ORPH_CA: f64 = 0.7; // скорость эксп-затухания после Ca > 1
                    (-LAMBDA_ORPH_CA * (ca - 1.0)).exp() // плавное затухание
                };
                // 2) «стабильность» по Is (низкая нестабильность = высокий коэффициент)
                let stability = 1.0 - logistic(is, LAMBDA_ORPH, TAU_ORPH);
                lack_of_use * stability
            }

            // Суммарная сила зависимостей Ds
            let ds: f64 = coupling_metrics
                .dependency_strength
                .values()
                .map(|&w| w as f64)
                .sum();

            // Средняя сила одной зависимости  S̄
            let s_bar = if ce == 0.0 { ds } else { ds / ce.max(1.0) };

            // Фактор циклов (=1, если циклов нет)
            let cycle_factor = if let Some(cycle_data) = &coupling_metrics.cycles {
                if let Some(cycle_id) = cycle_data.cycle_id {
                    // Используем размер конкретного цикла (SCC)
                    1.0 + all_cycles[cycle_id].size as f64
                } else {
                    1.0
                }
            } else {
                1.0
            };

            // ---------- 2. Абсолютный штраф ----------
            let abs_score = c_tot
                + K_SLOPE * ((s_bar / ALPHA_DS) + 1.0).ln()
                + K_CYCLE * cycle_factor.ln()
                + penalty_instability(instability)
                + K_KEY * key_risk(ca, instability)
                + K_ORPH * orphan_risk(ca, instability);

            abs_score
        }
    }
}

pub(crate) fn get_file_extension(file_name: &str) -> Option<String> {
    Path::new(file_name)
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
}

pub(crate) fn is_code_file(file_name: &str) -> bool {
    matches!(
        get_file_extension(file_name).as_deref(),
        Some("ts") | Some("js") | Some("tsx") | Some("jsx")
    )
}

pub(crate) fn is_test_file(file_name: &str) -> bool {
    file_name.contains("test.")
        || file_name.contains("spec.")
        || file_name.contains("tests/")
        || file_name.contains("__tests__/")
}

#[cfg(test)]
mod tests;

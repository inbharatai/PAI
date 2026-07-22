package com.unoone.agent.core.agent

import com.unoone.agent.core.model.ToolCall
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive

/**
 * Routes commands through deterministic parsers before invoking the model.
 *
 * Deterministic routing handles commands that don't need model reasoning:
 * - Wake commands ("uno on", "uno start", "listen")
 * - Language commands ("hindi mein bolo", "speak in english")
 * - Blind mode commands ("start blind", "stop blind")
 * - Simple app launches ("open whatsapp", "open gmail")
 * - Accessibility shortcuts ("go home", "go back", "read screen", "scroll down")
 *
 * Only when NO deterministic match is found does the orchestrator invoke the model pipeline.
 * This saves inference time, reduces model load, and eliminates hallucinated tool calls
 * for commands that have a single obvious interpretation.
 *
 * Each parser returns a [DeterministicResult]:
 * - [DeterministicResult.Matched] — a tool call was determined without the model.
 * - [DeterministicResult.NoMatch] — the command needs model reasoning.
 */
object DeterministicIntentRouter {

    /**
     * Result of attempting deterministic command routing.
     */
    sealed class DeterministicResult {
        /**
         * A deterministic match was found. The command can be executed without model inference.
         * [call] is the resolved tool call. [intent] is the classified intent for logging/analytics.
         */
        data class Matched(val call: ToolCall, val intent: String) : DeterministicResult()

        /** No deterministic match — the command should go to the model pipeline. */
        data object NoMatch : DeterministicResult()
    }

    /**
     * Route a command through deterministic parsers.
     *
     * The order matters: more specific patterns are checked first to avoid false matches.
     * For example, "open whatsapp and send message to Rahul" should NOT match the
     * simple app-launch pattern — it needs model reasoning for the compound intent.
     *
     * @param normalizedCommand the language-normalized, cleaned transcript from [LanguageNormalizer].
     * @return [DeterministicResult.Matched] if a deterministic route was found,
     *   [DeterministicResult.NoMatch] if the command needs model reasoning.
     */
    fun route(normalizedCommand: String): DeterministicResult {
        val lower = normalizedCommand.lowercase().trim()

        // ── Wake commands (no model, no tool) ──────────────────────────────
        if (isWakeCommand(lower)) {
            return DeterministicResult.NoMatch // Wake is handled by the voice layer, not tools
        }

        // ── Language commands ───────────────────────────────────────────────
        val langResult = routeLanguageCommand(lower)
        if (langResult != null) return langResult

        // ── Blind mode commands ─────────────────────────────────────────────
        val blindResult = routeBlindModeCommand(lower)
        if (blindResult != null) return blindResult

        // ── Simple app launches ─────────────────────────────────────────────
        val appResult = routeAppLaunch(lower)
        if (appResult != null) return appResult

        // ── Accessibility shortcuts (no model needed) ─────────────────────
        val accessResult = routeAccessibilityShortcut(lower)
        if (accessResult != null) return accessResult

        // ── Simple voice fast-replies ──────────────────────────────────────
        val fastReplyResult = routeVoiceFastReply(lower)
        if (fastReplyResult != null) return fastReplyResult

        // ── No deterministic match ─────────────────────────────────────────
        return DeterministicResult.NoMatch
    }

    // ── Wake commands ────────────────────────────────────────────────────

    private val WAKE_PHRASES = setOf(
        "uno on", "uno start", "hey uno", "ok uno", "listen",
        "uno suno", "uno shuru", "hello uno"
    )

    private fun isWakeCommand(lower: String): Boolean =
        WAKE_PHRASES.any { lower == it || lower.startsWith("$it ") }

    // ── Language commands ────────────────────────────────────────────────

    private val LANGUAGE_COMMANDS = mapOf(
        "speak in english" to "en",
        "answer in english" to "en",
        "in english" to "en",
        "speak in hindi" to "hi",
        "hindi mein bolo" to "hi",
        "हिंदी में बोलो" to "hi",
        "speak in tamil" to "ta",
        "speak in bengali" to "bn",
        "speak in telugu" to "te",
        "speak in kannada" to "kn",
        "speak in malayalam" to "ml"
    )

    private fun routeLanguageCommand(lower: String): DeterministicResult? {
        for ((phrase, lang) in LANGUAGE_COMMANDS) {
            if (lower.contains(phrase)) {
                return DeterministicResult.Matched(
                    call = ToolCall("speak_response", JsonObject(mapOf(
                        "text" to JsonPrimitive("Language switched to $lang")
                    ))),
                    intent = "LANGUAGE_SWITCH"
                )
            }
        }
        return null
    }

    // ── Blind mode commands ──────────────────────────────────────────────

    private fun routeBlindModeCommand(lower: String): DeterministicResult? {
        return when {
            lower.contains("start blind") || lower.contains("blind mode on") ||
                lower.contains("andha mode shuru") -> DeterministicResult.Matched(
                ToolCall("speak_response", JsonObject(mapOf(
                    "text" to JsonPrimitive("Blind aid mode activated")
                ))),
                intent = "BLIND_MODE_ON"
            )
            lower.contains("stop blind") || lower.contains("blind mode off") ||
                lower.contains("andha mode band") -> DeterministicResult.Matched(
                ToolCall("speak_response", JsonObject(mapOf(
                    "text" to JsonPrimitive("Blind aid mode deactivated")
                ))),
                intent = "BLIND_MODE_OFF"
            )
            else -> null
        }
    }

    // ── App launches ────────────────────────────────────────────────────

    private val APP_LAUNCHES = mapOf(
        "open whatsapp" to "com.whatsapp",
        "open gmail" to "com.google.android.gm",
        "open google" to "com.google.android.googlequicksearchbox",
        "open maps" to "com.google.android.apps.maps",
        "open youtube" to "com.google.android.youtube",
        "open calendar" to "com.google.android.calendar",
        "open chrome" to "com.android.chrome",
        "open settings" to "com.android.settings",
        "open camera" to "com.android.camera",
        "open phone" to "com.google.android.dialer",
        "open play store" to "com.android.vending",
        "open clock" to "com.google.android.deskclock"
    )

    private fun routeAppLaunch(lower: String): DeterministicResult? {
        // Only match simple, single-intent app launches.
        // Compound commands ("open whatsapp and send message") should fall through to the model.
        for ((phrase, _) in APP_LAUNCHES) {
            if (lower == phrase || lower == "$phrase app" || lower.startsWith("$phrase ") && !lower.contains(" and ") && !lower.contains(" send ") && !lower.contains(" message ")) {
                val packageName = APP_LAUNCHES[phrase]!!
                return DeterministicResult.Matched(
                    ToolCall("open_app", JsonObject(mapOf("package_name" to JsonPrimitive(packageName)))),
                    intent = "APP_LAUNCH"
                )
            }
        }
        return null
    }

    // ── Accessibility shortcuts ─────────────────────────────────────────

    private fun routeAccessibilityShortcut(lower: String): DeterministicResult? {
        return when {
            lower == "go home" || lower == "go home please" -> DeterministicResult.Matched(
                ToolCall("go_home", JsonObject(emptyMap())), "ACCESSIBILITY"
            )
            lower == "go back" || lower == "go back please" -> DeterministicResult.Matched(
                ToolCall("go_back", JsonObject(emptyMap())), "ACCESSIBILITY"
            )
            lower == "read screen" || lower == "read the screen" || lower == "what's on screen" ||
                lower == "what is on screen" || lower == "what's on my screen" -> DeterministicResult.Matched(
                ToolCall("read_screen", JsonObject(emptyMap())), "ACCESSIBILITY"
            )
            lower == "scroll down" || lower == "scroll down please" -> DeterministicResult.Matched(
                ToolCall("scroll", JsonObject(mapOf("direction" to JsonPrimitive("down")))), "ACCESSIBILITY"
            )
            lower == "scroll up" || lower == "scroll up please" -> DeterministicResult.Matched(
                ToolCall("scroll", JsonObject(mapOf("direction" to JsonPrimitive("up")))), "ACCESSIBILITY"
            )
            lower == "open notifications" || lower == "show notifications" -> DeterministicResult.Matched(
                ToolCall("open_notifications", JsonObject(emptyMap())), "ACCESSIBILITY"
            )
            lower == "open recents" || lower == "show recents" || lower == "recent apps" -> DeterministicResult.Matched(
                ToolCall("open_recents", JsonObject(emptyMap())), "ACCESSIBILITY"
            )
            else -> null
        }
    }

    // ── Voice fast replies ──────────────────────────────────────────────

    private val FAST_REPLIES = mapOf(
        "thank you" to "You're welcome!",
        "thanks" to "You're welcome!",
        "stop" to "Okay, stopping.",
        "cancel" to "Cancelled.",
        "never mind" to "Okay, no problem.",
        "yes" to "Okay.",
        "no" to "Okay.",
        "ok" to "Okay."
    )

    private fun routeVoiceFastReply(lower: String): DeterministicResult? {
        val reply = FAST_REPLIES[lower.trim()] ?: return null
        return DeterministicResult.Matched(
            ToolCall("speak_response", JsonObject(mapOf("text" to JsonPrimitive(reply)))),
            intent = "VOICE_FAST_REPLY"
        )
    }
}
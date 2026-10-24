(function () {
    "use strict";

    var storageKey = "soroban-cookbook-language";
    var defaultLanguage = "en";
    var languages = [
        { code: "en", label: "English", status: "available" },
        { code: "es", label: "Spanish", status: "English fallback" },
        { code: "fr", label: "French", status: "English fallback" }
    ];

    function getSavedLanguage() {
        try {
            return window.localStorage.getItem(storageKey);
        } catch (error) {
            return null;
        }
    }

    function saveLanguage(language) {
        try {
            window.localStorage.setItem(storageKey, language);
        } catch (error) {
            // The selector still works when browser storage is unavailable.
        }
    }

    function addLanguageSelector() {
        var toolbar = document.querySelector("#menu-bar .right-buttons");
        if (!toolbar) {
            return;
        }

        var wrapper = document.createElement("label");
        wrapper.className = "language-selector";

        var select = document.createElement("select");
        select.setAttribute("aria-label", "Documentation language");
        select.title = "Documentation language";

        var fallbackStatus = document.createElement("span");
        fallbackStatus.className = "language-fallback-status";
        fallbackStatus.setAttribute("aria-live", "polite");

        languages.forEach(function (language) {
            var option = document.createElement("option");
            option.value = language.code;
            option.textContent = language.label + " - " + language.status;
            select.appendChild(option);
        });

        var savedLanguage = getSavedLanguage();
        var knownLanguage = languages.some(function (language) {
            return language.code === savedLanguage;
        });
        select.value = knownLanguage ? savedLanguage : defaultLanguage;
        if (!knownLanguage && savedLanguage !== null) {
            saveLanguage(defaultLanguage);
        }

        function updateFallbackStatus() {
            var selectedLanguage = languages.find(function (language) {
                return language.code === select.value;
            });
            var usesFallback = selectedLanguage && selectedLanguage.code !== defaultLanguage;
            fallbackStatus.textContent = usesFallback ? "English fallback" : "";
            document.documentElement.lang = defaultLanguage;
        }

        updateFallbackStatus();
        select.addEventListener("change", function () {
            saveLanguage(select.value);
            updateFallbackStatus();
        });

        wrapper.appendChild(select);
        wrapper.appendChild(fallbackStatus);
        toolbar.insertBefore(wrapper, toolbar.firstChild);
    }

    if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", addLanguageSelector);
    } else {
        addLanguageSelector();
    }
})();
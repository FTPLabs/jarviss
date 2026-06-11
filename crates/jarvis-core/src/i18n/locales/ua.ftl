# APP INFO
app-name = FTPDev Assistant
app-description = Голосовий асистент

# TRAY MENU
tray-restart = Перезапустити
tray-settings = Налаштування
tray-exit = Вихід
tray-tooltip = FTPDev Voice Assistant
tray-language = Мова
tray-voice = Голос
tray-wake-word = Двигун wake-word
tray-noise-suppression = Шумопридушення
tray-vad = Визначення голосу (VAD)
tray-gain-normalizer = Нормалізація гучності

# HEADER
header-commands = КОМАНДИ
header-settings = НАЛАШТУВАННЯ

# SEARCH
search-placeholder = Введіть команду вручну або скажіть «ФТП» ...

# MAIN PAGE
assistant-not-running = АСИСТЕНТ НЕ ЗАПУЩЕНО
assistant-offline-hint = Налаштувати можна без запуску.
btn-start = ЗАПУСТИТИ
btn-starting = ЗАПУСК...

# STATUS
status-disconnected = Відключено
status-standby = Очікування
status-listening = Слухаю...
status-processing = Обробка...

# STATS
stats-microphone = МІКРОФОН
stats-neural-networks = НЕЙРОМЕРЕЖІ
stats-resources = РЕСУРСИ
stats-system-default = Системний
stats-not-selected = Не обрано
stats-loading = Завантаження...

# FOOTER
footer-author = FTPDev
footer-telegram = GitHub репозиторій
footer-github = Github репозиторій проекту
footer-support = Підтримати проект

# SETTINGS
settings-title = Налаштування
settings-general = Основні
settings-devices = Пристрої
settings-neural-networks = Нейромережі
settings-audio = Аудіо
settings-recognition = Розпізнавання
settings-about = Про програму
settings-language = Мова
settings-microphone = Мікрофон
settings-microphone-desc = Його буде слухати асистент.
settings-mic-default = За замовчуванням (Система)
settings-voice = Голос асистента
settings-voice-desc =
    Не всі команди працюють з усіма звуковими пакетами.
    Натисніть, щоб прослухати.
settings-wake-word-engine = Двигун активації
settings-wake-word-desc = Оберіть нейромережу для розпізнавання активаційної фрази.
settings-stt-engine = Розпізнавання мови
settings-intent-engine = Визначення наміру
settings-intent-engine-desc = Оберіть нейромережу для розпізнавання команд.
settings-noise-suppression = Шумопридушення
settings-noise-suppression-desc = Зменшує фоновий шум.
settings-vad = Визначення голосу (VAD)
settings-vad-desc = Пропускає тишу, економить CPU.
settings-gain-normalizer = Нормалізація гучності
settings-gain-normalizer-desc = Автоматично регулює рівень гучності.
settings-api-keys = API Ключі
settings-save = Зберегти
settings-cancel = Скасувати
settings-back = Назад
settings-enabled = Увімкнено
settings-disabled = Вимкнено

# settings - beta notice
settings-beta-title = БЕТА версія!
settings-beta-desc = Частина функцій може працювати некоректно.
settings-beta-feedback = Повідомляйте про знайдені баги в
settings-beta-bot = GitHub Issues
settings-open-logs = Відкрити папку з логами

# settings - picovoice
settings-attention = Увага!
settings-picovoice-warning = Ця нейромережа працює не у всіх!
settings-picovoice-waiting = Ми чекаємо офіційного патча від розробників.
settings-picovoice-key-desc = Введіть свій ключ Picovoice. Він видається безкоштовно при реєстрації в
settings-picovoice-key = Ключ Picovoice

# settings - vosk
settings-auto-detect = Авто-визначення
settings-vosk-model = Модель розпізнавання мови (Vosk)
settings-vosk-model-desc =
    Оберіть модель Vosk для розпізнавання мови.
    Завантажити моделі: https://alphacephei.com/vosk/models
settings-models-not-found = Моделі не знайдено
settings-models-hint = Помістіть моделі Vosk у папку resources/vosk

# settings - openai
settings-openai-key = Ключ OpenAI
settings-openai-not-supported = ChatGPT наразі не підтримується.

# COMMANDS PAGE
commands-title = Команди
commands-search = Пошук команд...
commands-count = { $count } команд
commands-wip-title = [404] Цей розділ ще в розробці!
commands-wip-desc = Тут буде список команд + повноцінний редактор.
commands-wip-follow = Слідкуйте за оновленнями в
commands-wip-channel = GitHub репозиторії

# ERRORS
error-generic = Сталася помилка
error-connection = Помилка підключення
error-not-found = Не знайдено

# NOTIFICATIONS
notification-saved = Налаштування збережено!
notification-error = Помилка
notification-assistant-started = Асистент запущено
notification-assistant-stopped = Асистент зупинено

# SLOTS EXTRACTION
settings-slot-engine = Вилучення параметрів
settings-slot-engine-desc = Вилучає параметри з голосових команд.
settings-gliner-model = Модель GLiNER ONNX
settings-gliner-model-desc =
    Оберіть варіант моделі.
    Квантизовані моделі (int8, uint8) швидші, але менш точні.
settings-gliner-models-hint = Моделі GLiNER не знайдено.

# ETC
search-error-not-running = Асистент не запущено
search-error-failed = Не вдалося виконати команду
settings-no-voices = Голоси не знайдено

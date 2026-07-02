# Чиста калькулятор 2.0 — извлечённый инвентарь (из распакованного exe)

Источник: `input/Чиста калькулятор 2.0.exe` (Delphi, UPX 1.24 со скремблированным заголовком).
Распаковано эмуляцией стаба (unipacker/unicorn) → `calc_unpacked.exe` (~1 МБ), строки в cp1251.

## Архитектура (по именам классов)
- Движок выражений: `TReckoner` / `TRCKParcer` / `TRecConst` / `TRCKParams`
- Исключения: `ERCKParserError`, `ERCKSyntaxError`, `ERCKVariableError`, `ERCKFunctionError`
- Крипто-бэкенд: **DCPcrypt** (open-source Delphi lib) — отсюда все шифры/хеши
- Токены парсера: ПУСТО, ЗНАЧЕНИЕ, ОДИНОЧНЫЙ ОПЕРАНД, ОПЕРАНД, РАЗДЕЛИТЕЛЬ ПАРАМЕТРОВ,
  КОНЕЦ ВЫРАЖЕНИЯ, ШЕСТНАДЦАТИРИЧНОЕ ЧИСЛО, ДВОИЧНОЕ ЧИСЛО

## Циклы
Ключевые слова: `while`, `loop`, `FORCE`. Сообщения: "Расчет в цикле...",
"Цикл завершен (проходов: N)", "Цикл продолжается длительное время. Остановить расчет?"

## Пользовательские функции / псевдонимы
`Function`, `Alias`, `Func(`, `Alias(`, `PARAM`. Регистрация: 'Функция "x" зарегистрированна',
'Псевдоним "x" зарегистрирован'.

## Настройки (.ini)
fWidth, fHeight, fTop, fLeft, fState, StoreText, OnTop, Function, Alias, Interface

## Встроенные функции (извлечено)

### Математика / тригонометрия
Sin Cos Tan Sqrt Sqr Ln Log Exp Abs Pi Int Frac Trunc Round Ceil Floor
ArcSin ArcCos ArcTan ArcSinH ArcCosH ArcTanH SinH CosH TanH Cotan
DegToRad RadToDeg CycleToRad RadToCycle

### Системы счисления / преобразования
RimToDec (римские) OctToDec StrToHex HexToStr DecToBase BaseToDec
Алфавит основания: 0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ (до base36)

### Битовые
BitTest BitSet BitToggle

### Строки
AnsiPos Copy Length StringReplace Trim TrimLeft TrimRight
UpperCase LowerCase AnsiUpperCase AnsiLowerCase
CompareStr CompareText AnsiCompareStr AnsiCompareText AnsiSameStr AnsiSameText
Ord Chr Pos  PerW PerS

### Дата/время и форматирование
Date Time Now
IntToStr StrToInt FloatToStr StrToFloat
DateToStr StrToDate TimeToStr StrToTime DateTimeToStr StrToDateTime
FormatDateTime FormatFloat

### Файлы
FileToStr StrToFile

### Хеши (DCPcrypt)
MD2 MD4 MD5 SHA1 SHA256 SHA384 SHA512
Haval128 Haval160 Haval192 Haval224 Haval256
Tiger128 Tiger160 Tiger192 RipeMD128 RipeMD160 Gost Adler32 CRC32

### Шифры (DCPcrypt, пары _E/_D = encrypt/decrypt)
Blowfish Cast128 Cast256 DES DES3 Ice ThinIce Ice2 IDEA MARS
Misty1 Rijndael Serpent TEA Twofish

## Сообщения об ошибках (семантика парсера)
"Синтаксическая ошибка", "Неправильный синтаксис выражения", "Неправильный синтаксис функции",
"Неправильное сравнение", "Неизвестный символ", "Неизвестная переменная 'x'",
"Неизвестная функция 'x'", "Функция 'x': неправильные параметры",
"Ожидалось X, но встретилось Y", "Ожидался конец комментариям", "Ошибка в строке комментариев"

## Прочее
Результат = ; "Значение переменной присвоено"; булевы: True/False, Light/Sound, On/Off;
"Нажмите ENTER для выполнения расчета"; заголовок окна "Калькулятор".

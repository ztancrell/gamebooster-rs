# FeedBack.exe notes (user feedback form)

Static only. Analysed copy: `sgb-extract/carved/pe_01d2eb6c_3839032.bin` (3,839,032 bytes; sha256
prefix `4434d92a562156bb`). Method: `strings`.

User-initiated form. Fields: e-mail (`UserMail`, remembered), category (`reqtype`: Suggestion,
Report, System, Temperature, Others; reasons like "Network Acceleration failure", "No Internet
connection", "Need more servers", "Account issue"), free text, optional screenshot. Sent to
`https://stats.gameboosta.com/smgbfeedback/feedback.php` together with licence tier fields
(`SVIP EXP`, `ACSVIP`, `signmail`, `trdmail` = signed-in / third-party account e-mail). On failure
it offers a web form or `mailto:`. It can also launch BugReport.exe (`/bugreport /smgb`) to attach
logs. Nothing here runs without the user opening the form, apart from the shared crash hook.

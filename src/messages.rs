pub static WELCOME_MESSAGE: &'static str = r"Willkommen Mensch!

Ich bin's, der remoteDeckel_bot.

Zusammen können wir saufen UND unsere Lieblingskneipe supporten.

Bestell' einfach deine Biers bei mir und ich schreib' sie auf deinen Deckel.

Du bestimmst wieviel du pro Getränk spenden möchtest.

Wenn OberkanteUnterlippe erreicht ist, gib mir Bescheid, um den 'Schaden' zu begleichen:
- Dann wird dein Deckel genullt
- und deine Spende übermittelt

Und keine Sorge. Wenn der Durst doch größer war als es die Haushaltskasse erlaubt. Du kannst jederzeit den Spendenbetrag reduzieren oder die ganze Zeche prellen.

Na dann, Prost!";

pub static TERMS: &'static str = r"NUTZUNGSBEDINGUNGEN:

Leistungen des Betreibers von remoteDeckel_bot:
Dieser Chat-Bot erbringt keine Leistungen für den Nutzer, außer dass er die vom Nutzer beauftragte Spendensumme an die Bochumer Gaststätte 'Li Buddah' weiterleitet.
Dafür verwendet der Chatbot, das von Telegram zur Verfügung gestellte Interface, welches die Zahlung über mit den Online-Zahlungs-Provider Stripe abwickelt.
Weder der Betreiber des remoteDeckel_bot noch Telegram haben jemals Einblick in die Kreditkartendaten des Nutzers. Die Kreditakrtendaten werden ausschließlich von Stripe für die Zahlung der Spende verarbeitet und verwendet.

Wie setzt sich die Spendensumme zusammen:
Die Spendensumme setzt sich zusammen aus dem Getränkzähler-Wert, mulitpliziert mit dem Betrag, den der Nutzer pro Einheit über die vom remoteDeckel_bot angebotenen Auswahlmöglichkeiten, festgelegt hat. Der Gertränkezähler entspricht der Anzahl der Klicks, die ein Nutzer auf dem Button 'Bring mir ein Bier' Button getätigt hat und die daraufhin vom remotDeckel_bot mit 'Ich schreibe es auf deinen Deckel' bestätigt worden sind.
Der Spendenbetrag kann nicht in vollem Umfang weitergeleitet werden, da der Stripe eine Gebühr pro Transaktion erhebt, die Stripe vom Spendenbetrag einbehält.
Der Gesamtbetrag der Spende ist der Endbetrag. Alle Gebühren (die Strip-Transaktionsgebühren) sind darin enthalten.

Verpflichtungen des Nutzers:
Der Nutzer darf den remoteDeckel_bot ausschließlich im Rahmen der von Telegram möglichen Interaktionen verwenden. Sollte dem Nutzer auffallen, dass der remoteDeckel_bot oder Telegram selbst eine Sicherheitslücke aufweist, ist dies dem Betreiber unverzüglich anzuzeigen. Ein Missbrauch einer etwaaigen Umgehung der eigentlichen, gemeinnützigen Intention des Bots ist untersagt.
Der Nutzer is zu keinem Zeitpunkt, vor einer Zahlungsbestätigung dazu verpflichtet eine Zahlung zu tätigen. Jeder Betrag, der über den remoteDecke_bot gesammelt wird ist zunächst unverbindlich, löschbar und freiwillig. Eine bereits angewiesene Spendenzahlung, für die der Checkout-Prozess abgeschlossen wurde ist verbindlich, spätesstens aber wenn dei Zahlung von Stripe verarbeitet wurde und Transaktionskosten verursacht wurden.
Der Nutzer kann zu jedem Zeitpunkt seine gesammelten Daten löschen. Dies gilt nicht für Zahlungsprotokolle, da diese von Stripe verwaltet werden und dort nachgehalten werden.
Der Nutzer ist sich bewusst, dass seine Spende, um die von Stripe (Online-Zahlungs-Provider) ehobene Gebühr gemindert wird. Die Gebühren können auf der Website von Stripe eingesehen werden: https://stripe.com/de/pricing.
Der Nutzer ist verantwortlich den finalen Spendenpreis auf der Rechnung von Stripe zu überprüfen. Wenn diese im Chatfentster bestätigt wird, gilt die Zahlung als bestätigt und kann nicht zurückgezogen werden.

Verpflichtungen des Betreibers:
Der Betreiber ermöglicht über den remoteDeckel_bot, dass der Nutzer eine Spende an die Gaststätte 'Li Buddah' tätigen kann. Dafür verpflichtet sich der Betreiber, selbst keine Gebühren oder anderweitigen Abschläge einzubehalten. Der Betreiber übernimmt nicht die Bearbeitungsgebühren des Online-Zahlungs-Providers Stripe. Der Betreiber übernimmt jedoch alle anderen, für den Betriebe des remoteDeckel_bot notwendigen  Betriebskosten, wie Hosting der Applikationssoftware und der Datenbank.
Der Betreiber garantiert nicht für die Erreichbarkeit des remoteDeckel_bot. Der Betreiber behält sich das Recht vor, den remoteDeckel_bot jederzeit abzuschalten, womit dieser nicht mehr erreichbar ist. Fehler in der Software oder Sicherheitprobleme von denen der Betreiber Kenntnis erlangt, wird der Betreiber im Rahmen seiner Fähigkeiten, so schnell wie möglich beheben. Sollte eine Fehlerbehebung nicht innerhalb eines angemessenen Zeitrahmens möglich sein, wird der remoteDeckel_bot abgeschaltet. Eine Information der Nutzer wird nicht garantiert.
Vor der Einblendung der Stripe-Rechnung gibt der remoteDeckel_bot eine Zahlungsübersicht als Chatnachricht, in der der Netto-Spendenbetrag und die Stripe-Gebühr separat aufgelistet werden. Diese Auflistung gilt lediglich als Orientierung. Der Betreiber übernimmt keine Haftung für Abweichungen in dieser Auflistung.
Der Betreiber garantiert nicht, dass Interaktionen mit dem Bot korrekt verarbeitet werden. Sollten zum Beispiel clicks nicht registriert werden, wodurch der Getränkezähler oder der Getränkepreis von der eigentlichen Nutzer-Intention abweicht, so ist dies nicht zu beanstanden. Auch die Verarbeitung von Zahlungsanweisungen wird nicht garantiert. Es wird lediglich versichert, dass bereits getätigte Zahlungen, die von Stripe erfasst wurden und genehmigt wurden, weitergeleitet werden.";

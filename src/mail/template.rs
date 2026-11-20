use reqwest::Url;
use std::fmt::Write;

pub(super) struct MessageText {
    plain: String,
    html: String,
}

impl MessageText {
    pub(super) fn parse(input: &str) -> Option<Self> {
        let mut plain = String::new();
        let mut html = String::new();
        let mut tags = Vec::new();
        let mut rest = input;
        while let Some((text, tail)) = rest.split_once('<') {
            plain.push_str(text);
            html.push_str(&lines(text));
            let Some((tag, next)) = tail.split_once('>') else {
                rest = &rest[text.len()..];
                break;
            };
            let original = &tail[..=tag.len()];
            match tag {
                "b" | "strong" | "i" | "em" | "u" => {
                    tags.push(tag);
                    let _ = write!(html, "<{tag}>");
                }
                "/b" | "/strong" | "/i" | "/em" | "/u"
                    if tags.last().is_some_and(|open| *open == &tag[1..]) =>
                {
                    tags.pop();
                    let _ = write!(html, "<{tag}>");
                }
                "br" | "br/" | "br /" => {
                    plain.push('\n');
                    html.push_str("<br>");
                }
                _ => {
                    plain.push('<');
                    plain.push_str(original);
                    html.push_str("&lt;");
                    html.push_str(&lines(original));
                }
            }
            rest = next;
        }
        plain.push_str(rest);
        html.push_str(&lines(rest));
        for tag in tags.into_iter().rev() {
            let _ = write!(html, "</{tag}>");
        }
        (!plain.trim().is_empty()).then_some(Self { plain, html })
    }
}

pub struct Details {
    pub description: Option<String>,
    pub location: String,
    pub directions: Url,
    pub timezone: String,
    pub dates: String,
    pub confirmed: bool,
    pub website: Option<Url>,
    pub calendar: Option<String>,
    pub notes: String,
}

const ATTRIBUTION: &str = "Geoapify · © OpenStreetMap contributors";

impl Details {
    fn date_label(&self, lang: &str) -> String {
        let key = if self.confirmed {
            "ui.confirmedDate"
        } else {
            "ui.dates"
        };
        rust_i18n::t!(key, locale = lang).into_owned()
    }
}

pub fn plain(
    lang: &str,
    kind: &str,
    headline: &str,
    title: &str,
    intro: &str,
    url: &str,
    details: Option<&Details>,
    message: Option<&MessageText>,
) -> String {
    let mut body = String::new();
    if kind == "confirmed" {
        let _ = writeln!(body, "{headline}");
    }
    let _ = write!(body, "{intro}\n\n{title}\n\n{url}\n");
    if let Some(message) = message {
        let _ = writeln!(body, "\n{}", message.plain);
    }
    if let Some(d) = details {
        if let Some(description) = &d.description {
            let _ = writeln!(body, "\n{description}");
        }
        let _ = write!(
            body,
            "\n{}\n{}\n{}: {}\n\n{}: {}\n{}: {}\n",
            d.date_label(lang),
            d.dates,
            rust_i18n::t!("ui.timezone", locale = lang),
            d.timezone,
            rust_i18n::t!("ui.location", locale = lang),
            d.location,
            rust_i18n::t!("mail.directions", locale = lang),
            d.directions,
        );
        if let Some(website) = &d.website {
            let _ = writeln!(
                body,
                "{}: {website} ({ATTRIBUTION})",
                rust_i18n::t!("mail.venueWebsite", locale = lang)
            );
        }
        if let Some(calendar) = &d.calendar {
            let _ = writeln!(
                body,
                "\n{}: {calendar}",
                rust_i18n::t!("ui.addCalendar", locale = lang)
            );
        }
        if !d.notes.is_empty() {
            let _ = writeln!(body, "\n{}", d.notes.trim_end());
        }
    }
    if message.is_none() {
        let _ = writeln!(body, "\n{}", rust_i18n::t!("mail.private", locale = lang));
    }
    body
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn lines(text: &str) -> String {
    escape(text).replace('\n', "<br>")
}

fn link(label: &str, url: &str) -> String {
    format!(
        r#"<a href="{}" style="color:#244c35;font-weight:bold;text-decoration:underline">{}</a>"#,
        escape(url),
        escape(label)
    )
}

fn facts(lang: &str, d: &Details) -> String {
    let description = d
        .description
        .as_deref()
        .filter(|text| !text.trim().is_empty())
        .map(|text| format!(r#"<p style="margin:0 0 20px">{}</p>"#, lines(text)))
        .unwrap_or_default();
    let directions = link(
        &rust_i18n::t!("mail.directions", locale = lang),
        d.directions.as_str(),
    );
    let website = d
        .website
        .as_ref()
        .map(|url| {
            format!(
                r#"<br>{}<br><span style="font-size:12px;color:#4d564a">{ATTRIBUTION}</span>"#,
                link(
                    &rust_i18n::t!("mail.venueWebsite", locale = lang),
                    url.as_str()
                )
            )
        })
        .unwrap_or_default();
    let calendar = d
        .calendar
        .as_ref()
        .map(|url| {
            format!(
                r#"<p style="margin:14px 0 0">{}</p>"#,
                link(&rust_i18n::t!("ui.addCalendar", locale = lang), url)
            )
        })
        .unwrap_or_default();
    let notes = if d.notes.is_empty() {
        String::new()
    } else {
        format!(
            r#"<p style="margin:20px 0 0;padding:14px 16px;border-radius:12px;background:#f0eaff">{}</p>"#,
            lines(d.notes.trim_end())
        )
    };
    format!(
        r##"{description}
<table role="presentation" width="100%" cellspacing="0" cellpadding="0" border="0">
  <tr><td bgcolor="#eef8e4" style="padding:14px 16px;border-radius:12px;color:#223423">
    <strong style="font-size:14px">📅 {date_label}</strong><br>{dates}<br><span style="font-size:13px;color:#4d564a">{timezone}</span>
  </td></tr>
  <tr><td height="8"></td></tr>
  <tr><td bgcolor="#fff0db" style="padding:14px 16px;border-radius:12px;color:#223423">
    <strong style="font-size:14px">📍 {location_label}</strong><br>{location}<br>{directions}{website}
  </td></tr>
</table>
{calendar}{notes}"##,
        date_label = escape(&d.date_label(lang)),
        dates = lines(&d.dates),
        timezone = escape(&format!(
            "{}: {}",
            rust_i18n::t!("ui.timezone", locale = lang),
            d.timezone
        )),
        location_label = escape(&rust_i18n::t!("ui.location", locale = lang)),
        location = lines(&d.location),
    )
}

pub fn html(
    lang: &str,
    kind: &str,
    headline: &str,
    title: &str,
    intro: &str,
    url: &str,
    details: Option<&Details>,
    message: Option<&MessageText>,
) -> String {
    let (banner, accent, icon) = match kind {
        "confirmed" => ("#d9f59d", "#fff0b2", "🎉"),
        "cancelled" => ("#f7e2e1", "#f7e2e1", ""),
        "changed" | "open" => ("#ffefae", "#ffd2bc", "✦"),
        "vote" => ("#e9dfff", "#d9f59d", "✓"),
        "message" => ("#e9dfff", "#ffd2bc", "💌"),
        _ => ("#dcefd4", "#ffeaaf", "✦"),
    };
    let action = match kind {
        "admin" => "mail.manageEvent",
        "vote" => "ui.edit",
        _ => "mail.openEvent",
    };
    let (direction, align) = if lang == "ar" {
        ("rtl", "right")
    } else {
        ("ltr", "left")
    };
    let facts = details
        .map(|details| facts(lang, details))
        .unwrap_or_default();
    let note = message.map(|message| format!(
        r#"<p style="margin:0 0 16px;padding:14px 16px;border-radius:12px;background:#fff0db">{}</p>"#,
        message.html
    )).unwrap_or_default();
    let privacy = if message.is_none() {
        format!(
            r#"<p style="margin:12px 0 0;color:#4d564a;font-size:12px">{}</p>"#,
            escape(&rust_i18n::t!("mail.private", locale = lang))
        )
    } else {
        String::new()
    };
    format!(
        r##"<!doctype html>
<html lang="{lang}" dir="{direction}">
<head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>{headline}</title></head>
<body style="margin:0;padding:0;background:#f4f6ef;color:#182923">
<table role="presentation" width="100%" cellspacing="0" cellpadding="0" border="0" bgcolor="#f4f6ef">
<tr><td align="center" style="padding:16px 10px 28px">
<!--[if mso]><table role="presentation" width="560" align="center"><tr><td><![endif]-->
<table role="presentation" width="100%" cellspacing="0" cellpadding="0" border="0" dir="{direction}" style="max-width:560px;font-family:Arial,sans-serif;line-height:1.55;text-align:{align};overflow-wrap:anywhere">
  <tr><td bgcolor="{banner}" style="border-top:5px solid {accent};border-radius:16px 16px 0 0;padding:16px 22px 18px;color:#182923">
    <div style="font-size:19px;font-weight:bold;letter-spacing:-0.6px">frizzi<span style="color:#467138">.</span></div>
    <h1 style="margin:10px 0 0;font-size:26px;line-height:1.22">{headline} {icon}</h1>
  </td></tr>
  <tr><td bgcolor="#ffffff" style="padding:20px 22px 24px;color:#182923;border-radius:0 0 16px 16px">
    <h2 style="margin:0 0 8px;font-size:21px;line-height:1.3">{title}</h2>
    <p style="margin:0 0 20px;font-size:16px">{intro}</p>
    {note}
    {facts}
    <table role="presentation" cellspacing="0" cellpadding="0" border="0" style="margin-top:20px">
      <tr><td bgcolor="#244c35" style="border-radius:10px;mso-padding-alt:0">
        <a href="{url}" style="display:inline-block;border:1px solid #244c35;padding:12px 18px;border-radius:10px;color:#ffffff;font-size:15px;font-weight:bold;text-decoration:none">{action}</a>
      </td></tr>
    </table>
    {privacy}
  </td></tr>
</table>
<!--[if mso]></td></tr></table><![endif]-->
</td></tr>
</table>
</body>
</html>"##,
        lang = escape(lang),
        headline = escape(headline),
        title = escape(title),
        intro = lines(intro),
        url = escape(url),
        action = escape(&rust_i18n::t!(action, locale = lang)),
    )
}

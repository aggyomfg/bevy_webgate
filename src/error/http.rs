use axum::{http::StatusCode, response::Response};
use bevy_app::{App, Plugin};
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::prelude::*;
use std::collections::HashMap;

pub struct HttpErrorPlugin;

impl Plugin for HttpErrorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HttpErrorResponses>();
    }
}

fn create_error_html(code: &str, title: &str, message: &str) -> String {
    format!(
        r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{code} - {title}</title>
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Poppins:wght@300;400;500;600;700&display=swap" rel="stylesheet">
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        body {{
            font-family: 'Poppins', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
            background: #0d1117;
            color: #e6edf3;
            line-height: 1.6;
            min-height: 100vh;
            display: flex;
            align-items: center;
            justify-content: center;
        }}
        .container {{
            text-align: center;
            max-width: 800px;
            padding: 2rem;
            background: #161b22;
            border-radius: 12px;
            border: 1px solid #30363d;
            box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
        }}
        .nav-section {{
            display: flex;
            align-items: center;
            justify-content: center;
            gap: 0.75rem;
            margin-bottom: 3rem;
            padding-bottom: 1.5rem;
            border-bottom: 1px solid #30363d;
        }}
        .nav-brand {{
            display: flex;
            align-items: center;
            gap: 0.75rem;
        }}
        .logo {{
            width: 100px;
            height: 40px;
            flex-shrink: 0;
        }}
        .logo img {{
            width: 100%;
            height: 100%;
            object-fit: contain;
        }}
        .nav-title {{
            font-size: 1.5rem;
            font-weight: 700;
            color: #e6edf3;
            letter-spacing: 0.1em;
        }}
        .nav-subtitle {{
            color: #7d8590;
            font-weight: 500;
            font-size: 0.9rem;
            margin-left: 1rem;
        }}
        .error-section {{
            margin: 2rem 0;
        }}
        .error-code {{
            font-size: clamp(4rem, 8vw, 8rem);
            font-weight: 600;
            margin: 0 0 1rem 0;
            background: linear-gradient(135deg, #e6edf3 0%, #ff6b35 100%);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            background-clip: text;
            line-height: 1;
        }}
        .error-title {{
            font-size: 2rem;
            font-weight: 600;
            color: #e6edf3;
            margin-bottom: 1rem;
        }}
        .error-message {{
            font-size: 1.1rem;
            color: #7d8590;
            margin-bottom: 2rem;
            font-weight: 400;
            line-height: 1.6;
        }}
        .back-button {{
            background: linear-gradient(135deg, #ff6b35 0%, #ff8c42 100%);
            color: white;
            border: none;
            padding: 0.875rem 2rem;
            font-size: 1rem;
            font-weight: 500;
            border-radius: 8px;
            cursor: pointer;
            transition: all 0.3s ease;
            font-family: inherit;
            text-decoration: none;
            display: inline-block;
        }}
        .back-button:hover {{
            transform: translateY(-2px);
            box-shadow: 0 8px 25px rgba(255, 107, 53, 0.4);
        }}
        .footer {{
            margin-top: 2rem;
            padding-top: 1.5rem;
            border-top: 1px solid #30363d;
            color: #7d8590;
            font-size: 0.9rem;
        }}
        .footer strong {{
            color: #e6edf3;
        }}
        @media (max-width: 768px) {{
            .container {{
                margin: 1rem;
                padding: 1.5rem;
            }}
            .nav-section {{
                flex-direction: column;
                gap: 0.5rem;
                text-align: center;
            }}
            .nav-brand {{
                flex-direction: column;
                gap: 0.5rem;
            }}
            .nav-subtitle {{
                margin-left: 0;
            }}
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="nav-section">
            <div class="nav-brand">
                <div class="logo">
                    <img src="static/bevy_logo_dark.svg" alt="Bevy Logo" />
                </div>
            </div>
            <div class="nav-subtitle">WebGate</div>
        </div>
        
        <div class="error-section">
            <h1 class="error-code">{code}</h1>
            <h2 class="error-title">{title}</h2>
            <p class="error-message">{message}</p>
            <button class="back-button" onclick="history.back()">Go Back</button>
        </div>
        
        <div class="footer">
            <p>Powered by <strong>Bevy WebGate</strong> - A refreshingly simple web server built in Rust</p>
        </div>
    </div>
</body>
</html>
    "#,
        code = code,
        title = title,
        message = message
    )
}

#[derive(Clone, Deref, DerefMut, Resource)]
pub struct HttpErrorResponses {
    responses: HashMap<StatusCode, String>,
}

impl HttpErrorResponses {
    pub fn get_response(&self, status: StatusCode) -> Option<&String> {
        self.responses.get(&status)
    }

    pub fn get_response_or_default(&self, status: StatusCode) -> String {
        self.responses.get(&status).cloned().unwrap_or_else(|| {
            create_error_html(
                &status.as_u16().to_string(),
                status.canonical_reason().unwrap_or("Error"),
                "An error occurred.",
            )
        })
    }

    pub fn create_response(&self, status: StatusCode) -> Response {
        let html = self.get_response_or_default(status);
        Response::builder()
            .status(status)
            .header("Content-Type", "text/html")
            .body(html.to_string().into())
            .unwrap_or_default()
    }
}

impl Default for HttpErrorResponses {
    fn default() -> Self {
        let mut responses = HashMap::new();

        responses.insert(
            StatusCode::BAD_REQUEST,
            create_error_html(
                "400",
                "Bad Request",
                "Sorry, we couldn't process your request.",
            ),
        );

        responses.insert(
            StatusCode::UNAUTHORIZED,
            create_error_html(
                "401",
                "Unauthorized",
                "Sorry, you are not authorized to access this resource.",
            ),
        );

        responses.insert(
            StatusCode::FORBIDDEN,
            create_error_html(
                "403",
                "Forbidden",
                "Sorry, you don't have permission to access this resource.",
            ),
        );

        responses.insert(
            StatusCode::NOT_FOUND,
            create_error_html("404", "Not Found", "Sorry, we couldn't find that page."),
        );

        responses.insert(
            StatusCode::INTERNAL_SERVER_ERROR,
            create_error_html(
                "500",
                "Internal Server Error",
                "Something went wrong on our end.",
            ),
        );

        responses.insert(
            StatusCode::SERVICE_UNAVAILABLE,
            create_error_html(
                "503",
                "Service Unavailable",
                "The service is temporarily unavailable.",
            ),
        );

        Self { responses }
    }
}

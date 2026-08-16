use atspi::{
    connection::{set_session_accessibility, AccessibilityConnection},
    proxy::{
        accessible::{AccessibleProxy, ObjectRefExt},
        proxy_ext::ProxyExt,
    },
    CoordType, Interface, InterfaceSet,
};

/// Connect to AT-SPI and print the accessibility tree.
pub async fn inspect() -> Result<(), Box<dyn std::error::Error>> {
    // Setting this property is idempotent. It also lets applications which watch
    // the session status turn on their accessibility support before we connect.
    set_session_accessibility(true).await?;

    let connection = AccessibilityConnection::new().await?;
    let root = connection.root_accessible_on_registry().await?;

    traverse(root, &connection, 0).await;
    Ok(())
}

async fn traverse(
    accessible: AccessibleProxy<'_>,
    connection: &AccessibilityConnection,
    depth: usize,
) {
    let indent = "  ".repeat(depth);
    let role = match accessible.get_role().await {
        Ok(role) => format!("{role:?}"),
        Err(error) => {
            warn(depth, "could not read role", &error);
            "Unknown".to_owned()
        }
    };
    let name = match accessible.name().await {
        Ok(name) => name.escape_default().collect::<String>(),
        Err(error) => {
            warn(depth, "could not read name", &error);
            String::new()
        }
    };

    let interfaces = match accessible.get_interfaces().await {
        Ok(interfaces) => Some(interfaces),
        Err(error) => {
            warn(depth, "could not read interfaces", &error);
            None
        }
    };

    let interface_text = interfaces
        .as_ref()
        .map_or_else(|| "unknown".to_owned(), format_interfaces);
    let action = interfaces.as_ref().map_or("unknown", |set| {
        if set.contains(Interface::Action) {
            "true"
        } else {
            "false"
        }
    });

    print!("{indent}{role} \"{name}\" interfaces=[{interface_text}] action={action}");

    if interfaces
        .as_ref()
        .is_some_and(|set| set.contains(Interface::Component))
    {
        match screen_bounds(&accessible, depth).await {
            Some((x, y, width, height)) => {
                print!(" x={x} y={y} w={width} h={height}");
            }
            None => {
                print!(" bounds=unavailable");
            }
        }
    }
    println!();

    let children = match accessible.get_children().await {
        Ok(children) => children,
        Err(error) => {
            warn(depth, "could not read children", &error);
            return;
        }
    };

    for child in children {
        match child.into_accessible_proxy(connection.connection()).await {
            Ok(child_proxy) => {
                Box::pin(traverse(child_proxy, connection, depth + 1)).await;
            }
            Err(error) => warn(depth + 1, "could not open child", &error),
        }
    }
}

async fn screen_bounds(
    accessible: &AccessibleProxy<'_>,
    depth: usize,
) -> Option<(i32, i32, i32, i32)> {
    let proxies = match accessible.proxies().await {
        Ok(proxies) => proxies,
        Err(error) => {
            warn(depth, "could not create interface proxies", &error);
            return None;
        }
    };
    let component = match proxies.component().await {
        Ok(component) => component,
        Err(error) => {
            warn(depth, "could not open Component interface", &error);
            return None;
        }
    };

    match component.get_extents(CoordType::Screen).await {
        Ok(bounds) => Some(bounds),
        Err(error) => {
            warn(depth, "could not read screen bounds", &error);
            None
        }
    }
}

fn format_interfaces(interfaces: &InterfaceSet) -> String {
    interfaces
        .iter()
        .map(interface_name)
        .collect::<Vec<_>>()
        .join(", ")
}

fn interface_name(interface: Interface) -> &'static str {
    match interface {
        Interface::Accessible => "Accessible",
        Interface::Action => "Action",
        Interface::Application => "Application",
        Interface::Cache => "Cache",
        Interface::Collection => "Collection",
        Interface::Component => "Component",
        Interface::Document => "Document",
        Interface::DeviceEventController => "DeviceEventController",
        Interface::DeviceEventListener => "DeviceEventListener",
        Interface::EditableText => "EditableText",
        Interface::Hyperlink => "Hyperlink",
        Interface::Hypertext => "Hypertext",
        Interface::Image => "Image",
        Interface::Registry => "Registry",
        Interface::Selection => "Selection",
        Interface::Socket => "Socket",
        Interface::Table => "Table",
        Interface::TableCell => "TableCell",
        Interface::Text => "Text",
        Interface::Value => "Value",
    }
}

fn warn(depth: usize, message: &str, error: &dyn std::fmt::Display) {
    eprintln!("{}warning: {message}: {error}", "  ".repeat(depth));
}

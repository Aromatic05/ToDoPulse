use super::*;
use anyhow::Result;
use notify_rust::Notification;



pub fn notify_desktop(
  title: &str, 
  content: &str, 
  icon_path: Option<String>
) -> Result<()> {
    
    let mut notification = Notification::new();
    notification
    .summary(title)
    .body(content);
    if let Some(icon) = icon_path {
        notification.icon(&icon);
    }
    notification.show()?;
    Ok(())
}

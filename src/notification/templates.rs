#![allow(dead_code)]

pub fn welcome_email(username: &str, verify_url: &str) -> (String, String) {
    let subject = format!("Welcome to YSH, {}!", username);
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8"></head>
<body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; max-width: 600px; margin: 0 auto; padding: 20px; background: #0a0a0a; color: #e0e0e0;">
  <div style="background: linear-gradient(135deg, #1a1a2e, #16213e); padding: 40px; border-radius: 12px; text-align: center;">
    <h1 style="color: #00d4ff; font-size: 32px; margin: 0;">Welcome to YSH</h1>
    <p style="color: #888; font-size: 14px; margin-top: 8px;">Your journey begins now</p>
  </div>
  <div style="padding: 30px 0;">
    <h2 style="color: #ffffff;">Hey {username},</h2>
    <p style="color: #ccc; line-height: 1.6;">
      Your account has been created successfully. Please verify your email to unlock all features.
    </p>
    <div style="text-align: center; margin: 30px 0;">
      <a href="{verify_url}" style="background: #00d4ff; color: #000; padding: 14px 32px; border-radius: 8px; text-decoration: none; font-weight: bold; display: inline-block;">
        Verify Email
      </a>
    </div>
    <p style="color: #666; font-size: 12px;">
      If you didn't create this account, ignore this email.
    </p>
  </div>
  <div style="border-top: 1px solid #333; padding-top: 20px; color: #555; font-size: 12px; text-align: center;">
    YSH Platform &mdash; Connecting People
  </div>
</body>
</html>"#,
        username = username,
        verify_url = verify_url
    );
    (subject, html)
}

pub fn verify_email(username: &str, verify_url: &str) -> (String, String) {
    let subject = "Verify your YSH email".to_string();
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8"></head>
<body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; max-width: 600px; margin: 0 auto; padding: 20px; background: #0a0a0a; color: #e0e0e0;">
  <div style="background: linear-gradient(135deg, #1a1a2e, #16213e); padding: 40px; border-radius: 12px; text-align: center;">
    <h1 style="color: #00d4ff; font-size: 28px; margin: 0;">Email Verification</h1>
  </div>
  <div style="padding: 30px 0;">
    <h2 style="color: #ffffff;">Hey {username},</h2>
    <p style="color: #ccc; line-height: 1.6;">
      Click below to verify your email address.
    </p>
    <div style="text-align: center; margin: 30px 0;">
      <a href="{verify_url}" style="background: #00d4ff; color: #000; padding: 14px 32px; border-radius: 8px; text-decoration: none; font-weight: bold; display: inline-block;">
        Verify Now
      </a>
    </div>
    <p style="color: #666; font-size: 12px;">
      This link expires in 24 hours.
    </p>
  </div>
</body>
</html>"#,
        username = username,
        verify_url = verify_url
    );
    (subject, html)
}

pub fn reset_password(username: &str, reset_url: &str) -> (String, String) {
    let subject = "Reset your YSH password".to_string();
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8"></head>
<body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; max-width: 600px; margin: 0 auto; padding: 20px; background: #0a0a0a; color: #e0e0e0;">
  <div style="background: linear-gradient(135deg, #2e1a1a, #3e1616); padding: 40px; border-radius: 12px; text-align: center;">
    <h1 style="color: #ff6b6b; font-size: 28px; margin: 0;">Password Reset</h1>
  </div>
  <div style="padding: 30px 0;">
    <h2 style="color: #ffffff;">Hey {username},</h2>
    <p style="color: #ccc; line-height: 1.6;">
      We received a request to reset your password. Click below to set a new one.
    </p>
    <div style="text-align: center; margin: 30px 0;">
      <a href="{reset_url}" style="background: #ff6b6b; color: #000; padding: 14px 32px; border-radius: 8px; text-decoration: none; font-weight: bold; display: inline-block;">
        Reset Password
      </a>
    </div>
    <p style="color: #666; font-size: 12px;">
      If you didn't request this, your account may be compromised. Contact support immediately.
    </p>
  </div>
</body>
</html>"#,
        username = username,
        reset_url = reset_url
    );
    (subject, html)
}

pub fn gift_received(username: &str, from_user: &str, gift_name: &str) -> (String, String) {
    let subject = format!("{} sent you a gift!", from_user);
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8"></head>
<body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; max-width: 600px; margin: 0 auto; padding: 20px; background: #0a0a0a; color: #e0e0e0;">
  <div style="background: linear-gradient(135deg, #1a2e1a, #163e16); padding: 40px; border-radius: 12px; text-align: center;">
    <h1 style="color: #00ff88; font-size: 28px; margin: 0;">You received a gift!</h1>
  </div>
  <div style="padding: 30px 0;">
    <h2 style="color: #ffffff;">Hey {username},</h2>
    <p style="color: #ccc; line-height: 1.6;">
      <strong>{from_user}</strong> sent you a <strong>{gift_name}</strong>!
    </p>
    <p style="color: #ccc; line-height: 1.6;">
      Check your wallet to see your new gift.
    </p>
  </div>
</body>
</html>"#,
        username = username,
        from_user = from_user,
        gift_name = gift_name
    );
    (subject, html)
}

pub fn call_missed(username: &str, caller: &str) -> (String, String) {
    let subject = format!("Missed call from {}", caller);
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8"></head>
<body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; max-width: 600px; margin: 0 auto; padding: 20px; background: #0a0a0a; color: #e0e0e0;">
  <div style="background: linear-gradient(135deg, #2e2e1a, #3e3e16); padding: 40px; border-radius: 12px; text-align: center;">
    <h1 style="color: #ffcc00; font-size: 28px; margin: 0;">Missed Call</h1>
  </div>
  <div style="padding: 30px 0;">
    <h2 style="color: #ffffff;">Hey {username},</h2>
    <p style="color: #ccc; line-height: 1.6;">
      <strong>{caller}</strong> tried to call you. You were offline at the time.
    </p>
    <p style="color: #ccc; line-height: 1.6;">
      Go online to receive calls and start earning!
    </p>
  </div>
</body>
</html>"#,
        username = username,
        caller = caller
    );
    (subject, html)
}

pub fn moment_liked(username: &str, liker: &str) -> (String, String) {
    let subject = format!("{} liked your moment", liker);
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8"></head>
<body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; max-width: 600px; margin: 0 auto; padding: 20px; background: #0a0a0a; color: #e0e0e0;">
  <div style="background: linear-gradient(135deg, #2e1a2e, #3e163e); padding: 40px; border-radius: 12px; text-align: center;">
    <h1 style="color: #ff66cc; font-size: 28px; margin: 0;">New Like</h1>
  </div>
  <div style="padding: 30px 0;">
    <h2 style="color: #ffffff;">Hey {username},</h2>
    <p style="color: #ccc; line-height: 1.6;">
      <strong>{liker}</strong> liked your moment!
    </p>
  </div>
</body>
</html>"#,
        username = username,
        liker = liker
    );
    (subject, html)
}

pub fn weekly_digest(username: &str, stats: &str) -> (String, String) {
    let subject = "Your YSH weekly digest".to_string();
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8"></head>
<body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; max-width: 600px; margin: 0 auto; padding: 20px; background: #0a0a0a; color: #e0e0e0;">
  <div style="background: linear-gradient(135deg, #1a1a2e, #0e2340); padding: 40px; border-radius: 12px; text-align: center;">
    <h1 style="color: #00d4ff; font-size: 28px; margin: 0;">Weekly Digest</h1>
    <p style="color: #888; font-size: 14px;">Your week in review</p>
  </div>
  <div style="padding: 30px 0;">
    <h2 style="color: #ffffff;">Hey {username},</h2>
    <div style="background: #1a1a2e; padding: 20px; border-radius: 8px; color: #ccc; line-height: 1.8;">
      {stats}
    </div>
    <p style="color: #ccc; line-height: 1.6; margin-top: 20px;">
      Keep growing your presence on YSH!
    </p>
  </div>
</body>
</html>"#,
        username = username,
        stats = stats
    );
    (subject, html)
}

/**
 * Independent TypeSpec authority for ORES Chat trust-surface policy.
 *
 * This slice projects route-owned authentication requirements to clients and
 * UIs. It never makes a UI selection an authorization fact and never carries
 * credentials or verified identity data.
 */
namespace Ores.Chat.TrustSurface.V1;

enum ChatSurface {
  public,
  user,
  admin,
}

model PublicChatSurfacePolicy {
  protocol: "ores.chat.trust_surface/v1";
  surface: "public";
}

model UserChatSurfacePolicy {
  protocol: "ores.chat.trust_surface/v1";
  surface: "user";
  required_realm: "ores-chat-customer";
  required_token_audience: "ores-chat-api";
}

model AdminChatSurfacePolicy {
  protocol: "ores.chat.trust_surface/v1";
  surface: "admin";
  required_realm: "ores-chat-admin";
  required_token_audience: "ores-chat-admin-api";
}

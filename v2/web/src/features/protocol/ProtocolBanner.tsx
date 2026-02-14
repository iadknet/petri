type ConnectionState = "connecting" | "reconnecting" | "connected" | "error";

interface ProtocolBannerProps {
  protocolVersion: string;
  protocolMismatch: boolean;
  connectionState: ConnectionState;
  errorMessage: string | null;
}

function bannerState({
  protocolMismatch,
  connectionState,
}: Pick<ProtocolBannerProps, "protocolMismatch" | "connectionState">): string {
  if (protocolMismatch) {
    return "protocol_mismatch";
  }
  return connectionState;
}

export function ProtocolBanner(props: ProtocolBannerProps) {
  const compatibility = props.protocolMismatch ? "version mismatch" : "compatible";
  const state = bannerState(props);

  return (
    <span>
      {props.protocolVersion} ({compatibility}) | {state}
      {props.errorMessage ? ` | ${props.errorMessage}` : ""}
    </span>
  );
}

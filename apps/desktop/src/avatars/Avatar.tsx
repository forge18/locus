import { deriveAvatar, normalizeAvatarStyle } from "./derive";
import { useAvatarStyle } from "./preferences";

export interface AvatarProps {
    seed: string;
    alt: string;
    avatarStyle?: string;
    class?: string;
    testId?: string;
}

/** Shared avatar image; callers choose only the stable seed and surface sizing class. */
export function Avatar(props: AvatarProps) {
    const selectedStyle = useAvatarStyle();
    const style = () =>
        normalizeAvatarStyle(props.avatarStyle ?? selectedStyle());

    return (
        <img
            class={props.class}
            src={deriveAvatar(style(), props.seed)}
            alt={props.alt}
            data-avatar-style={style()}
            data-testid={props.testId}
        />
    );
}

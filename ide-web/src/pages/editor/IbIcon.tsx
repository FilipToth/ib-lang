/// Where the icon is served from. A path of its own, not one relative to the
/// page: the editor is served under /app, where a relative path would be read
/// against whatever page is open. CRA fills PUBLIC_URL in from the `homepage`
/// in package.json.
export const ibIconSrc = `${process.env.PUBLIC_URL}/assets/ib.png`;

const IbIcon = () => {
    return (
        <img
            src={ibIconSrc}
            alt=""
            style={{
                width: "24px",
                height: "24px",
                // in a crowded row it would otherwise be squeezed out of shape
                flexShrink: 0,
                backgroundColor: "transparent",
            }}
        />
    );
};

export default IbIcon;

const IbIcon = () => {
    return (
        <img
            src="assets/ib.png"
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
import type { ReactNode } from "react";
import clsx from "clsx";
import Heading from "@theme/Heading";
import styles from "./styles.module.css";

type FeatureItem = {
    title: string;
    Svg: React.ComponentType<React.ComponentProps<"svg">>;
    description: ReactNode;
};

const FeatureList: FeatureItem[] = [
    {
        title: "Run & Debug",
        Svg: require("@site/static/img/code-1.svg").default,
        description: (
            <>
                Effortlessly run your IB Pseudocode on our servers and identify
                issues instantly with our comprehensive error diagnostics. Get
                clear feedback to refine your code with ease.
            </>
        ),
    },
    {
        title: "Learn Effectively",
        Svg: require("@site/static/img/book-1.svg").default,
        description: (
            <>
                Gain a deeper understanding of your code with our extensive
                documentation and practical code samples, designed to teach
                syntax and core concepts effectively.
            </>
        ),
    },
    {
        title: "Open-source",
        Svg: require("@site/static/img/cloud-check-circle.svg").default,
        description: (
            <>
                Effortlessly run the entire app locally using Docker Compose.
                Explore the fully open-source repository, available on GitHub
                for complete transparency.
            </>
        ),
    },
];

function Feature({ title, Svg, description }: FeatureItem) {
    return (
        <div className={clsx("col col--4")}>
            <div className="text--center">
                <Svg className={styles.featureSvg} role="img" />
            </div>
            <div className="text--center padding-horiz--md">
                <Heading as="h3">{title}</Heading>
                <p>{description}</p>
            </div>
        </div>
    );
}

export default function HomepageFeatures(): ReactNode {
    return (
        <section className={styles.features}>
            <div className="container">
                <div className="row">
                    {FeatureList.map((props, idx) => (
                        <Feature key={idx} {...props} />
                    ))}
                </div>
            </div>
        </section>
    );
}

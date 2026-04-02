import clsx from 'clsx';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';

import styles from './index.module.css';

function HomepageHeader() {
  const { siteConfig } = useDocusaurusContext();
  return (
    <header className={clsx('hero hero--primary', styles.heroBanner)}>
      <div className="container">
        <h1 className={clsx('hero__title', styles.heroTitle)}>
          🐾 {siteConfig.title}
        </h1>
        <p className="hero__subtitle">{siteConfig.tagline}</p>
        <div className={styles.buttons}>
          <Link
            className="button button--secondary button--lg"
            to="/docs/intro"
          >
            시작하기 →
          </Link>
          <Link
            className="button button--outline button--secondary button--lg"
            href="https://github.com/DevNergis/Capybara-Lang"
          >
            GitHub
          </Link>
        </div>
      </div>
    </header>
  );
}

const features = [
  {
    emoji: '🌏',
    title: '한국어 친화적',
    description:
      '변수명, 주석, 문자열 모두 한국어를 완벽하게 지원합니다. 동아시아 개발자를 위한 언어입니다.',
  },
  {
    emoji: '⚡',
    title: 'Rust 기반',
    description:
      'Rust로 구현되어 높은 성능과 메모리 안전성을 보장합니다.',
  },
  {
    emoji: '🎯',
    title: '독창적 문법',
    description:
      '세미콜론, 다양한 괄호, 화살표 블록(<-, ->)을 활용한 고유한 문법 구조를 가집니다.',
  },
];

function Feature({ emoji, title, description }) {
  return (
    <div className={clsx('col col--4', styles.feature)}>
      <div className={styles.featureEmoji}>{emoji}</div>
      <h3>{title}</h3>
      <p>{description}</p>
    </div>
  );
}

export default function Home() {
  const { siteConfig } = useDocusaurusContext();
  return (
    <Layout
      title={siteConfig.title}
      description="한국어 친화적 실험적 프로그래밍 언어 Capybara-Lang 문서"
    >
      <HomepageHeader />
      <main>
        <section className={styles.features}>
          <div className="container">
            <div className="row">
              {features.map((feature) => (
                <Feature key={feature.title} {...feature} />
              ))}
            </div>
          </div>
        </section>

        <section className={styles.codeSection}>
          <div className="container">
            <h2>Hello World</h2>
            <pre className={styles.codeBlock}>
              <code>{`<print:("안녕하세요, 세계!")>`}</code>
            </pre>
          </div>
        </section>
      </main>
    </Layout>
  );
}

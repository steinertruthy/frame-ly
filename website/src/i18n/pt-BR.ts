import type { Messages } from './index';

export const ptBR: Messages = {
  meta: {
    title: 'FrameLy — Gravação de tela, screenshots e transcrição no seu computador',
    description:
      'O FrameLy tira screenshots e grava a tela no macOS e no Windows, e depois transcreve, resume e organiza as gravações em tópicos com uma IA que pode rodar inteiramente no seu computador.',
    ogAlt: 'Janela do FrameLy mostrando uma gravação com transcrição, resumo e tópicos',
  },
  a11y: {
    skip: 'Pular para o conteúdo',
    menu: 'Abrir menu',
    close: 'Fechar menu',
    language: 'Idioma',
    home: 'Início do FrameLy',
    screenshotOf: 'Captura de tela do FrameLy:',
    playPause: 'Reproduzir ou pausar o vídeo de demonstração',
  },
  nav: {
    overview: 'Visão geral',
    features: 'Funcionalidades',
    ai: 'Inteligência Artificial',
    how: 'Como funciona',
    download: 'Download',
    cta: 'Baixar',
  },
  os: { macos: 'macOS', windows: 'Windows' },
  hero: {
    eyebrow: 'Gravação de tela · Screenshots · Transcrição',
    title: 'Capture. Grave.\nEntenda.',
    subtitle:
      'O FrameLy é um gravador de tela e ferramenta de screenshots rápido e discreto para macOS e Windows, que transforma suas gravações em transcrições, resumos e tópicos — com uma IA que pode rodar inteiramente no seu computador.',
    ctaFor: 'Baixar para {os}',
    ctaGeneric: 'Baixar o FrameLy',
    ctaSecondary: 'Veja como funciona',
    detected: 'Sistema detectado: {os}',
    notDetected: 'Escolha seu sistema abaixo',
    compat: 'macOS 14+ com Apple Silicon · Windows 10 22H2+ (x64)',
    version: 'Versão {version}',
    noRelease: 'Instaladores em preparação',
    free: 'Gratuito e de código aberto (MIT)',
  },
  intro: {
    eyebrow: 'Visão geral',
    title: 'Um app da captura ao significado.',
    body:
      'O FrameLy fica na barra de menus ou na bandeja do sistema. Tire um screenshot e anote direto na seleção, grave a tela com o áudio do sistema e o microfone, e deixe o FrameLy transcrever a gravação e destacar o que importa. Seus arquivos ficam nas pastas que você escolher.',
    points: [
      {
        title: 'Nativo e leve',
        body: 'Feito em Rust com as APIs de captura de cada sistema — ScreenCaptureKit no macOS, Windows.Graphics.Capture no Windows. Sem FFmpeg e sem extensão de navegador.',
      },
      {
        title: 'Teclado em primeiro lugar',
        body: 'Atalhos globais para todos os modos de captura, um painel rápido e linha de comando para Raycast, Alfred, Atalhos ou AutoHotkey.',
      },
      {
        title: 'A IA que você escolher',
        body: 'A transcrição sempre roda no seu computador. Resumos e tópicos podem usar modelos locais, o Apple Intelligence ou a sua própria chave de API na nuvem.',
      },
    ],
  },
  features: {
    eyebrow: 'Funcionalidades',
    title: 'Cinco ferramentas, um fluxo tranquilo.',
    benefit: 'Por que ajuda',
    items: {
      screenshot: {
        kicker: 'Screenshots',
        title: 'Selecione, anote e compartilhe — sem sair da seleção.',
        body: 'Capture a tela inteira, uma janela ou qualquer região. Setas, formas, texto, destaque, desfoque e recorte aparecem direto na seleção, com desfazer e refazer. Copie para a área de transferência ou salve na sua pasta.',
        benefit: 'Explique um bug ou um detalhe de design em segundos, com as partes sensíveis desfocadas.',
        alt: 'Editor de screenshots do FrameLy com ferramentas de anotação em volta de uma região selecionada',
      },
      recording: {
        kicker: 'Gravação de tela',
        title: 'Grave a tela, uma janela ou uma região — com som.',
        body: 'Misture o áudio do sistema e o microfone, pause e retome, e pare por uma pequena barra de gravação ou por atalho. H.264 ou HEVC, com contagem regressiva e tamanho estimado do arquivo antes de começar.',
        benefit: 'Tutoriais, demonstrações e registros de reuniões sem montar um estúdio.',
        alt: 'Tela inicial do FrameLy com atalhos de captura e gravação',
      },
      transcription: {
        kicker: 'Transcrição',
        title: 'Cada palavra, no seu computador.',
        body: 'O FrameLy transcreve gravações e áudios ou vídeos importados com modelos abertos de fala que rodam localmente, detecta o idioma automaticamente e guarda o tempo de cada palavra. Clique em qualquer trecho para ir àquele momento, edite o texto e exporte em TXT, SRT, VTT, Markdown ou PDF.',
        benefit: 'Gravações pesquisáveis e legendas prontas, sem enviar o áudio para lugar nenhum.',
        alt: 'Transcrição no FrameLy com player de vídeo e texto com marcações de tempo',
      },
      summaries: {
        kicker: 'Resumos',
        title: 'A reunião em um parágrafo.',
        body: 'Gere um resumo breve, equilibrado ou detalhado a partir da transcrição — com os modelos locais do FrameLy, com o Apple Intelligence em Macs compatíveis, ou com Anthropic ou OpenAI usando a sua chave.',
        benefit: 'Compartilhe as decisões e os próximos passos em vez de um vídeo de 40 minutos.',
        alt: 'Aba de resumo do FrameLy com pontos principais e próximos passos',
      },
      topics: {
        kicker: 'Tópicos',
        title: 'Um mapa do que foi dito.',
        body: 'O FrameLy organiza a transcrição em tópicos e subtópicos, cada um com seu momento na gravação. Abra um tópico para ler os pontos ou clique no horário para tocar aquele trecho.',
        benefit: 'Encontre a parte que você precisa numa gravação longa com um olhar.',
        alt: 'Aba de tópicos do FrameLy com tópicos e subtópicos marcados no tempo',
      },
    },
  },
  ai: {
    eyebrow: 'Inteligência Artificial',
    title: 'Três formas de pensar. Você decide onde.',
    subtitle:
      'A transcrição sempre acontece no seu computador. Para resumos e tópicos, escolha o mecanismo que combina com a gravação — e troque quando quiser nos Ajustes.',
    modes: {
      local: {
        name: 'FrameLy AI Local',
        label: 'No seu computador',
        tag: 'Padrão · neste computador',
        body: 'Modelos abertos baixados pelo app e executados no seu computador. Sem conta, sem chave e sem custo por uso. Depois que os modelos estão baixados, transcrição, resumos e tópicos funcionam sem internet.',
        points: [
          'Fala: 3 níveis de precisão (de 181 MB a 1 GB)',
          'Linguagem: 3 perfis (de 2,6 a 5,3 GB), sugeridos pela sua memória RAM',
          'Downloads verificados por SHA-256; pause, retome ou remova quando quiser',
          'O modo “Exclusivamente local” bloqueia qualquer conexão',
        ],
      },
      cloud: {
        name: 'Na nuvem',
        label: 'FrameLy AI Cloud · Anthropic · OpenAI',
        tag: 'Opcional · exige internet',
        body: 'Resumos e tópicos gerados em servidores, e não no seu computador. Só o texto da transcrição é enviado — nunca o áudio ou o vídeo — e só depois da sua autorização.',
        options: [
          { name: 'FrameLy AI Cloud', status: 'Ainda não disponível nesta versão', body: 'O serviço de nuvem do próprio FrameLy: gratuito, sem conta e sem chave, com limite diário por computador. Um provedor de IA parceiro gera o resultado. Ele já está no app, mas o serviço ainda não está no ar, por isso as versões publicadas o mostram como indisponível.' },
          { name: 'Anthropic (Claude) ou OpenAI', status: 'Disponível · pago, com sua chave', body: 'Use a sua própria chave de API. Vem desligado: ligue os provedores externos, adicione a chave e autorize cada provedor. O uso é cobrado na sua conta com eles.' },
        ],
        points: [
          'Chaves guardadas nas Chaves do macOS ou no Gerenciador de Credenciais do Windows',
          'O modo “Exclusivamente local” bloqueia todas as opções na nuvem',
        ],
      },
      apple: {
        name: 'Apple Intelligence',
        label: 'No seu Mac',
        tag: 'macOS 26 ou posterior',
        body: 'Num Mac com Apple Intelligence, o FrameLy pode escrever resumos e tópicos com o modelo da Apple que roda no próprio aparelho. Nada para baixar no FrameLy e nenhuma chave.',
        points: [
          'Exige o macOS 26 ou posterior',
          'Um Mac compatível com o Apple Intelligence, ativado nos Ajustes do Sistema',
          'Nos idiomas que o Apple Intelligence suporta',
          'Não disponível no Windows',
        ],
      },
    },
    table: {
      caption: 'Compare as opções de IA para resumos e tópicos',
      feature: 'Opção',
      rows: {
          availability: 'Disponibilidade',
          where: 'Onde processa',
          connection: 'Internet',
          platforms: 'Plataformas',
          hardware: 'Hardware',
          cost: 'Custo',
          setup: 'Configuração',
      },
      columns: {
        local: {
          name: 'FrameLy AI Local',
          availability: 'Disponível',
          where: 'No seu computador',
          connection: 'Só para baixar os modelos',
          platforms: 'macOS e Windows',
          hardware: 'Apple Silicon ou CPU x64 com AVX2; cerca de 3 a 6 GB de memória livre, conforme o perfil',
          cost: 'Gratuito',
          setup: 'Baixar os modelos no app (de 3 a 6,5 GB no total)',
        },
        cloud: {
          name: 'FrameLy AI Cloud',
          availability: 'Ainda não nas versões publicadas',
          where: 'Serviço do FrameLy e um provedor de IA parceiro',
          connection: 'Obrigatória',
          platforms: 'macOS e Windows',
          hardware: 'Qualquer computador que rode o FrameLy',
          cost: 'Gratuito, com limite diário',
          setup: 'Autorizar no primeiro uso; sem chave',
        },
        external: {
          name: 'Anthropic / OpenAI',
          availability: 'Disponível',
          where: 'Servidores da Anthropic ou da OpenAI',
          connection: 'Obrigatória',
          platforms: 'macOS e Windows',
          hardware: 'Qualquer computador que rode o FrameLy',
          cost: 'Pago por uso, ao provedor',
          setup: 'Ligar, adicionar a chave de API e autorizar o provedor',
        },
        apple: {
          name: 'Apple Intelligence',
          availability: 'macOS 26 ou posterior',
          where: 'No seu Mac',
          connection: 'Não precisa',
          platforms: 'macOS 26 ou posterior',
          hardware: 'Mac compatível com o Apple Intelligence',
          cost: 'Gratuito',
          setup: 'Ativar o Apple Intelligence nos Ajustes do Sistema',
        },
      },
      note: 'A transcrição em si sempre usa os modelos de fala locais do FrameLy, qualquer que seja o modo escolhido para resumos e tópicos.',
    },
    figure: 'O gerenciador de modelos nos Ajustes do FrameLy: instale, verifique ou remova cada modelo. Tamanhos e licenças vêm do próprio app.',
    figureAlt: 'Gerenciador de modelos do FrameLy com modelos de transcrição e de linguagem, tamanho, memória e licença',
  },
  how: {
    eyebrow: 'Como funciona',
    title: 'Do primeiro atalho ao resumo.',
    steps: [
      { title: 'Instale', body: 'Baixe o instalador do seu sistema e abra o FrameLy. Ele fica na barra de menus ou na bandeja do sistema.' },
      { title: 'Permita o acesso', body: 'Autorize a gravação de tela e, se quiser, o microfone. O FrameLy explica para que serve cada permissão.' },
      { title: 'Capture ou grave', body: 'Use um atalho ou o painel rápido para tirar um screenshot ou gravar a tela, uma janela ou uma região.' },
      { title: 'Escolha a IA', body: 'Baixe os modelos locais numa etapa só, ou escolha o Apple Intelligence ou a sua chave de nuvem.' },
      { title: 'Transcreva', body: 'Envie uma gravação ou qualquer áudio ou vídeo para Transcrições. Arquivos longos continuam de onde pararam.' },
      { title: 'Resuma e compartilhe', body: 'Gere o resumo e os tópicos, edite e exporte a transcrição em TXT, SRT, VTT, Markdown ou PDF.' },
    ],
  },
  uses: {
    eyebrow: 'Casos de uso',
    title: 'Feito para o trabalho de todo dia.',
    items: [
      { title: 'Relatar bugs', body: 'Um screenshot com uma seta e um e-mail desfocado, ou uma gravação curta dos passos.' },
      { title: 'Registro de reuniões', body: 'Grave uma chamada com áudio do sistema e microfone e receba o resumo e os próximos passos.' },
      { title: 'Tutoriais e demonstrações', body: 'Grave um passo a passo e exporte legendas em SRT ou VTT.' },
      { title: 'Aulas e palestras', body: 'Transforme uma gravação longa em tópicos para navegar entre eles.' },
      { title: 'Revisão de design', body: 'Aponte detalhes com formas e texto, direto na captura.' },
      { title: 'Conteúdo sensível', body: 'Mantenha tudo no computador com a IA local e o modo “Exclusivamente local”.' },
    ],
  },
  demo: {
    eyebrow: 'Veja na prática',
    title: 'Uma gravação real, transcrita e resumida pelo FrameLy.',
    caption: 'Capturado do FrameLy rodando no macOS. O áudio da reunião é voz sintetizada.',
  },
  download: {
    eyebrow: 'Download',
    title: 'Baixe o FrameLy.',
    subtitle: 'Gratuito para macOS e Windows. Escolha seu sistema — a escolha fica salva.',
    choose: 'Sistema operacional',
    detectedHint: 'Detectamos {os}.',
    notDetectedHint: 'Não conseguimos identificar seu sistema. Escolha um:',
    button: 'Baixar para {os}',
    version: 'Versão',
    released: 'Lançamento',
    arch: 'Arquitetura',
    format: 'Formato',
    size: 'Tamanho',
    checksum: 'SHA-256',
    checksums: 'Todos os checksums',
    requirements: 'Requisitos',
    install: 'Como instalar',
    previous: 'Versões anteriores',
    noPrevious: 'Ainda não há versões anteriores.',
    changelog: 'Novidades',
    allReleases: 'Todos os releases no GitHub',
    portable: 'Portátil (zip)',
    none: 'Ainda não há instalador publicado para {os}. Acompanhe o repositório para saber quando sair.',
    unavailableArch: 'Ainda não disponível',
    archNames: { arm64: 'Apple Silicon (arm64)', x64: 'x64 (Intel/AMD 64 bits)', universal: 'Universal' },
    formats: { dmg: 'Imagem de disco (.dmg)', exe: 'Instalador (.exe, NSIS)', msi: 'Instalador (.msi)', zip: 'Portátil (.zip)' },
    req: {
      macos: ['macOS 14 Sonoma ou posterior', 'Mac com Apple Silicon (M1 ou posterior)', 'Macs Intel: ainda sem versão', 'Recursos do Apple Intelligence: macOS 26 ou posterior'],
      windows: ['Windows 10 22H2 ou Windows 11', 'x64 (Intel ou AMD 64 bits) — ARM64: ainda sem versão', 'Microsoft Edge WebView2 (já vem no Windows 11; o instalador pode configurá-lo)', 'IA local: CPU com AVX2'],
    },
    steps: {
      macos: [
        'Abra o .dmg e arraste o FrameLy para Aplicativos.',
        'Esta versão ainda não é notarizada pela Apple. Na primeira vez, abra o FrameLy, vá em Ajustes do Sistema › Privacidade e Segurança e clique em “Abrir Mesmo Assim”.',
        'Permita a Gravação de Tela (e o Microfone, se for usar) quando o FrameLy pedir.',
      ],
      windows: [
        'Execute o instalador -setup.exe. Ele instala só para o seu usuário — sem precisar de administrador.',
        'O instalador ainda não tem assinatura de código, então o SmartScreen do Windows pode avisar. Clique em “Mais informações” e depois em “Executar assim mesmo”.',
        'O FrameLy abre na bandeja do sistema. Use o painel rápido ou Ctrl+Alt+Shift+Espaço.',
      ],
    },
    verify: 'Compare o SHA-256 do arquivo com o valor acima: shasum -a 256 <arquivo> no macOS, ou Get-FileHash <arquivo> no PowerShell.',
    windowsNote: 'Windows: a gravação de tela inteira foi testada em hardware real; gravação de janela e região, pausa, áudio e a IA local ainda estão sendo verificados em mais PCs com Windows.',
  },
  faq: {
    eyebrow: 'Perguntas frequentes',
    title: 'Dúvidas, respondidas.',
    more: 'Todas as perguntas',
    items: [
      {
        q: 'O FrameLy é gratuito?',
        a: 'Sim. O FrameLy é gratuito e de código aberto, sob a licença MIT. A IA local e o Apple Intelligence são gratuitos; os provedores de nuvem que você conectar com a sua chave cobram diretamente de você.',
      },
      {
        q: 'Meu áudio ou vídeo sai do meu computador?',
        a: 'Não para a transcrição: ela sempre roda no seu computador. Se você escolher um provedor de nuvem (Anthropic ou OpenAI) para resumos ou tópicos, só o texto da transcrição é enviado a esse provedor, depois da sua autorização.',
      },
      {
        q: 'O que é a FrameLy AI Cloud?',
        a: 'O serviço de nuvem gratuito do FrameLy para resumos e tópicos, sem conta e sem chave. O texto da transcrição é enviado a ele e um provedor de IA parceiro gera o resultado; conforme esse provedor, o conteúdo pode ser usado para melhorar os serviços dele, e o app avisa antes do uso. Ele já faz parte do app, mas ainda não está no ar, por isso as versões atuais o mostram como indisponível.',
      },
      {
        q: 'O FrameLy funciona offline?',
        a: 'Screenshots, gravação e, depois de baixar os modelos, transcrição, resumos e tópicos com a IA local funcionam sem internet. Baixar modelos e usar provedores de nuvem exigem conexão.',
      },
      {
        q: 'Quanto espaço os modelos de IA ocupam?',
        a: 'O modelo de fala ocupa de 181 MB a 1 GB, conforme a precisão, e o modelo de linguagem de 2,6 a 5,3 GB, conforme o perfil. O FrameLy sugere um perfil pela memória do seu computador, e você pode remover os modelos quando quiser.',
      },
      {
        q: 'Quais Macs suportam o Apple Intelligence no FrameLy?',
        a: 'Macs compatíveis com o Apple Intelligence rodando o macOS 26 ou posterior, com o Apple Intelligence ativado nos Ajustes do Sistema. O FrameLy em si roda no macOS 14 ou posterior.',
      },
      {
        q: 'O FrameLy funciona em Macs Intel ou no Windows ARM?',
        a: 'Ainda não. As versões atuais são para Macs com Apple Silicon e PCs com Windows 64 bits Intel/AMD.',
      },
      {
        q: 'Ele identifica quem está falando?',
        a: 'Ainda não. O reconhecimento de fala local do FrameLy não identifica os participantes.',
      },
      {
        q: 'Por que o macOS ou o Windows avisam na instalação?',
        a: 'Os instaladores ainda não são assinados com um Developer ID da Apple nem com um certificado de assinatura de código do Windows. Você pode conferir cada arquivo pelo checksum SHA-256 publicado na área de download e no GitHub.',
      },
      {
        q: 'Em quais idiomas o app está disponível?',
        a: 'A interface está em português (Brasil), inglês, espanhol, francês e alemão. A transcrição detecta automaticamente o idioma falado.',
      },
    ],
  },
  footer: {
    tagline: 'Gravação de tela, screenshots e transcrição no seu computador, para macOS e Windows.',
    product: 'Produto',
    resources: 'Recursos',
    legal: 'Legal',
    github: 'Código-fonte',
    releases: 'Releases',
    issues: 'Relatar um problema',
    changelog: 'Novidades',
    faq: 'Perguntas frequentes',
    privacy: 'Privacidade',
    terms: 'Termos de uso',
    license: 'Licença',
    rights: '© {year} FrameLy. Distribuído sob a licença MIT.',
    trademarks: 'Apple, macOS e Apple Intelligence são marcas da Apple Inc. Windows é marca da Microsoft. Claude e Anthropic são marcas da Anthropic. OpenAI é marca da OpenAI.',
  },
  pages: {
    pending: 'Rascunho — pendente de revisão jurídica. Este texto descreve como o FrameLy funciona hoje; não é aconselhamento jurídico.',
    updated: 'Última atualização: {date}',
    back: 'Voltar ao início',
    toc: 'Nesta página',
    notFound: { title: 'Página não encontrada', body: 'A página que você procura não existe ou foi movida.' },
    changelog: {
      title: 'Novidades',
      description: 'Todas as versões do FrameLy, com novidades, melhorias e correções.',
      intro: 'Todas as mudanças relevantes do FrameLy. As versões seguem o Versionamento Semântico.',
      added: 'Novidades',
      changed: 'Melhorias',
      fixed: 'Correções',
      known: 'Limitações conhecidas',
      assets: 'Downloads',
      release: 'Release no GitHub',
      empty: 'Ainda não há versões publicadas.',
    },
    faq: { title: 'Perguntas frequentes', description: 'Respostas sobre o FrameLy: preço, privacidade, modelos de IA, compatibilidade e instalação.' },
    license: {
      title: 'Licença',
      description: 'O FrameLy é código aberto sob a licença MIT.',
      intro: 'O código-fonte do FrameLy é distribuído sob a licença MIT. Os modelos de IA que o FrameLy baixa têm licenças próprias, listadas abaixo.',
      models: 'Modelos baixados pelo FrameLy',
      modelsBody: 'O reconhecimento de fala usa modelos Whisper (MIT) por meio do whisper.cpp. Resumos e tópicos com a IA local usam modelos de linguagem abertos das famílias Gemma e Qwen, distribuídos sob Apache 2.0. A licença de cada modelo também aparece no gerenciador de modelos do app.',
      thirdParty: 'Software de terceiros',
      thirdPartyBody: 'O FrameLy é feito com Tauri, Rust, Svelte, whisper.cpp e llama.cpp, entre outros projetos de código aberto, cada um com sua própria licença.',
      full: 'Texto completo da licença',
    },
    privacy: {
      title: 'Política de Privacidade',
      description: 'Como o FrameLy trata screenshots, gravações, transcrições e dados enviados a provedores de IA.',
      sections: [
        {
          h: 'Resumo',
          p: [
            'O FrameLy é um aplicativo de computador. Não tem conta, cadastro nem rastreadores de análise ou publicidade. Seus screenshots, gravações e transcrições ficam no seu computador, nas pastas que você escolher.',
            'Os dados só saem do seu computador quando você usa um recurso que precisa de internet: baixar modelos de IA, ou um provedor de IA em nuvem que você ligou e autorizou.',
          ],
        },
        {
          h: 'Screenshots e gravações',
          p: [
            'As capturas são salvas como arquivos nas pastas definidas nos Ajustes (por padrão, as pastas Imagens e Filmes/Vídeos). O FrameLy mantém um índice local delas para a biblioteca. O FrameLy não envia esses arquivos para lugar nenhum.',
          ],
        },
        {
          h: 'Transcrição (sempre local)',
          p: [
            'O reconhecimento de fala roda no seu computador com modelos baixados. Áudio e vídeo nunca são enviados a um servidor para transcrição. Transcrições, resumos e tópicos ficam num banco de dados local no seu computador, e você pode apagá-los no app.',
          ],
        },
        {
          h: 'IA local e Apple Intelligence',
          p: [
            'Com a IA local do FrameLy, resumos e tópicos são gerados no seu computador. Com o Apple Intelligence, são gerados pelo modelo da Apple no próprio aparelho, por meio do macOS; o FrameLy não envia nada a servidores da Apple para isso. As políticas da própria Apple se aplicam ao Apple Intelligence.',
          ],
        },
        {
          h: 'IA na nuvem (opcional)',
          p: [
            'A FrameLy AI Cloud é o serviço de nuvem opcional do próprio FrameLy. Ela ainda não está no ar, então as versões publicadas não podem usá-la. Quando estiver, e só depois da sua autorização, o FrameLy enviará a ela o texto da transcrição e a tarefa pedida, e um provedor de IA parceiro gerará o resultado. Seu computador é identificado por uma chave anônima criada nele — sem conta. O serviço não guarda o conteúdo: armazena as chaves anônimas e a contagem de uso diária. Conforme o plano do provedor, o conteúdo pode ser usado para melhorar os serviços dele; o app mostra isso antes do uso. Esta política será atualizada quando o serviço entrar no ar.',
            'Se você ligar os provedores externos, adicionar a sua chave de API e autorizar um provedor, o FrameLy envia o texto da transcrição (não o áudio nem o vídeo) e as instruções do resumo ou dos tópicos diretamente a esse provedor — Anthropic ou OpenAI. Valem os termos e a política de privacidade do provedor, inclusive sobre por quanto tempo ele guarda os dados. O uso é cobrado na sua conta com ele.',
            'As chaves de API e a chave anônima da FrameLy AI Cloud ficam nas Chaves do macOS ou no Gerenciador de Credenciais do Windows, e não em arquivos comuns. O ajuste “Exclusivamente local” bloqueia todas essas conexões.',
          ],
        },
        {
          h: 'Download de modelos',
          p: [
            'Ao baixar modelos de IA, seu computador se conecta aos servidores que os hospedam (como o Hugging Face). Esses servidores veem uma requisição de download comum, incluindo seu endereço IP. Cada arquivo é conferido por um hash SHA-256 conhecido.',
          ],
        },
        {
          h: 'Este site',
          p: [
            'Este site é estático e hospedado no GitHub Pages. Não usa cookies nem ferramentas de análise. Suas escolhas de idioma e de sistema operacional ficam apenas no armazenamento local do seu navegador. O GitHub pode tratar dados técnicos, como endereços IP, para servir o site e os downloads — veja a declaração de privacidade do GitHub.',
          ],
        },
        {
          h: 'Seu controle',
          p: [
            'Você pode apagar capturas, transcrições e modelos pelo app ou pelo sistema de arquivos quando quiser. Desinstalar o FrameLy remove o app; os arquivos das suas pastas de captura continuam lá até você apagá-los.',
          ],
        },
        {
          h: 'Contato',
          p: ['Dúvidas sobre privacidade: abra uma issue no repositório do FrameLy no GitHub.'],
        },
      ],
    },
    terms: {
      title: 'Termos de Uso',
      description: 'Os termos para usar o FrameLy e este site.',
      sections: [
        {
          h: 'O software',
          p: [
            'O FrameLy é oferecido gratuitamente sob a licença MIT. Você pode usá-lo, copiá-lo, modificá-lo e distribuí-lo conforme os termos dessa licença.',
          ],
        },
        {
          h: 'Sem garantia',
          p: [
            'O FrameLy é fornecido “no estado em que se encontra”, sem garantia de qualquer tipo. Transcrições, resumos e tópicos são gerados automaticamente e podem conter erros; confira informações importantes antes de confiar nelas.',
          ],
        },
        {
          h: 'Seu conteúdo e sua responsabilidade',
          p: [
            'Você é responsável pelo que captura, grava e processa. Garanta que tem o direito e, quando necessário, o consentimento das pessoas envolvidas para gravar telas, áudio e conversas, e para enviar transcrições a provedores de IA de terceiros.',
          ],
        },
        {
          h: 'Serviços de terceiros',
          p: [
            'Os provedores de IA em nuvem (Anthropic, OpenAI), o Apple Intelligence e os serviços que hospedam os modelos são operados por terceiros, com termos próprios. Usá-los com o FrameLy é opcional, e as cobranças são entre você e eles.',
          ],
        },
        {
          h: 'Downloads',
          p: [
            'Baixe o FrameLy apenas do repositório oficial no GitHub ou deste site, e confira o checksum SHA-256 publicado com cada release.',
          ],
        },
        {
          h: 'Alterações',
          p: ['Estes termos podem mudar conforme o FrameLy evolui. A data no topo indica a versão mais recente.'],
        },
      ],
    },
  },
};

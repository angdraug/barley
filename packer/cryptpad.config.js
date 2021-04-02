module.exports = {
    httpUnsafeOrigin: 'http://localhost:3000',
    httpSafeOrigin: "https://your-sandbox-domain.com",
    adminEmail: 'admin@your-main-domain.com',
    blockDailyCheck: true,

    filePath: '/var/lib/cryptpad/datastore/',
    archivePath: '/var/lib/cryptpad/data/archive',
    pinPath: '/var/lib/cryptpad/data/pins',
    taskPath: '/var/lib/cryptpad/data/tasks',
    blockPath: '/var/lib/cryptpad/block',
    blobPath: '/var/lib/cryptpad/blob',
    blobStagingPath: '/var/lib/cryptpad/data/blobstage',
    decreePath: '/var/lib/cryptpad/data/decrees',

    logPath: false,
    logToStdout: true,
    logLevel: 'warn',
    logFeedback: false,
    verbose: false,
};

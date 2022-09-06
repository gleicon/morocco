const algoliasearch = require('algoliasearch');

const stHost = {
  protocol: 'http',
  url: 'localhost:3000',
  accept: 1
};

const stHost2 = {
  protocol: 'http',
  url: 'localhost:3000',
  accept: 2
};

const client = algoliasearch("applicationId", "apiKey");

client.transporter.hosts = [stHost, stHost2];

//console.log(client.transporter.hosts);

const index = client.initIndex('poemas');

const objects = [
  {
    objectID: 1,
    title: 'El foo de la fuera',
    body: 'El fuero de la fuera fueron fuerar con pontito...'
  },
];

index.saveObjects(objects).then(({ objectIDs }) => {
  console.log(objectIDs);
}).catch(err => {
  console.log(JSON.stringify(err, null, 2));
});

index.search('fuera').then(({ hits }) => {
  console.log(hits);
}).catch(err => {
  console.log(JSON.stringify(err, null, 2));
});
